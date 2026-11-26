pub const NUM_FEATURES: usize = 1200;
pub const ACC_SIZE: usize = 256;
pub const L1_SIZE: usize = 32;
pub const L2_SIZE: usize = 32;

#[derive(Clone)]
pub struct NnueModel {
    pub feature_bias: [i16; ACC_SIZE],
    pub feature_weights: Vec<[i16; ACC_SIZE]>,
    pub l1_weights: [[i8; L1_SIZE]; ACC_SIZE],
    pub l1_bias: [i32; L1_SIZE],
    pub l2_weights: [[i8; L2_SIZE]; L1_SIZE],
    pub l2_bias: [i32; L2_SIZE],
    pub out_weights: [i8; L2_SIZE],
    pub out_bias: i32,
}

impl NnueModel {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("WNNU buffer too small for header".to_string());
        }
        if &bytes[0..4] != b"WNNU" {
            return Err("Invalid magic bytes in WNNU header".to_string());
        }

        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if version != 1 {
            return Err(format!("Unsupported WNNU version {}", version));
        }

        let mut offset = 8usize;

        let mut feature_bias = [0i16; ACC_SIZE];
        for b in feature_bias.iter_mut() {
            if offset + 2 > bytes.len() {
                return Err("Unexpected EOF reading feature_bias".to_string());
            }
            *b = i16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
            offset += 2;
        }

        let mut feature_weights = Vec::with_capacity(NUM_FEATURES);
        for _ in 0..NUM_FEATURES {
            let mut row = [0i16; ACC_SIZE];
            for w in row.iter_mut() {
                if offset + 2 > bytes.len() {
                    return Err("Unexpected EOF reading feature_weights".to_string());
                }
                *w = i16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
                offset += 2;
            }
            feature_weights.push(row);
        }

        let mut l1_weights = [[0i8; L1_SIZE]; ACC_SIZE];
        for row in l1_weights.iter_mut() {
            if offset + L1_SIZE > bytes.len() {
                return Err("Unexpected EOF reading l1_weights".to_string());
            }
            for (j, w) in row.iter_mut().enumerate() {
                *w = bytes[offset + j] as i8;
            }
            offset += L1_SIZE;
        }

        let mut l1_bias = [0i32; L1_SIZE];
        for b in l1_bias.iter_mut() {
            if offset + 4 > bytes.len() {
                return Err("Unexpected EOF reading l1_bias".to_string());
            }
            *b = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
        }

        let mut l2_weights = [[0i8; L2_SIZE]; L1_SIZE];
        for row in l2_weights.iter_mut() {
            if offset + L2_SIZE > bytes.len() {
                return Err("Unexpected EOF reading l2_weights".to_string());
            }
            for (j, w) in row.iter_mut().enumerate() {
                *w = bytes[offset + j] as i8;
            }
            offset += L2_SIZE;
        }

        let mut l2_bias = [0i32; L2_SIZE];
        for b in l2_bias.iter_mut() {
            if offset + 4 > bytes.len() {
                return Err("Unexpected EOF reading l2_bias".to_string());
            }
            *b = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
        }

        let mut out_weights = [0i8; L2_SIZE];
        if offset + L2_SIZE > bytes.len() {
            return Err("Unexpected EOF reading out_weights".to_string());
        }
        for (j, w) in out_weights.iter_mut().enumerate() {
            *w = bytes[offset + j] as i8;
        }
        offset += L2_SIZE;

        if offset + 4 > bytes.len() {
            return Err("Unexpected EOF reading out_bias".to_string());
        }
        let out_bias = i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());

        Ok(Self {
            feature_bias,
            feature_weights,
            l1_weights,
            l1_bias,
            l2_weights,
            l2_bias,
            out_weights,
            out_bias,
        })
    }

    #[inline]
    pub fn forward(&self, acc: &NnueAccumulator) -> f64 {
        let mut l1_act = self.l1_bias;
        for i in 0..ACC_SIZE {
            let val = acc.vals[i];
            let crelu = val.clamp(0, 127) as i32;
            if crelu > 0 {
                let row = &self.l1_weights[i];
                for j in 0..L1_SIZE {
                    l1_act[j] += crelu * (row[j] as i32);
                }
            }
        }

        let mut l2_act = self.l2_bias;
        for j in 0..L1_SIZE {
            let crelu = (l1_act[j] / 64).clamp(0, 127);
            if crelu > 0 {
                let row = &self.l2_weights[j];
                for k in 0..L2_SIZE {
                    l2_act[k] += crelu * (row[k] as i32);
                }
            }
        }

        let mut out = self.out_bias;
        for k in 0..L2_SIZE {
            let crelu = (l2_act[k] / 64).clamp(0, 127);
            if crelu > 0 {
                out += crelu * (self.out_weights[k] as i32);
            }
        }

        (out as f64) / 16.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NnueAccumulator {
    pub vals: [i16; ACC_SIZE],
}

impl NnueAccumulator {
    #[inline]
    pub fn new(model: &NnueModel) -> Self {
        Self {
            vals: model.feature_bias,
        }
    }

    #[inline]
    pub fn update_bitboard(&mut self, model: &NnueModel, bb: &crate::types::Bitboard, is_friendly: bool) {
        let base_feat = if is_friendly { 0 } else { 600 };
        for word_idx in 0..10 {
            let mut w = bb.0[word_idx];
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let cell = word_idx * 64 + bit;
                if cell < 600 {
                    let weights = &model.feature_weights[base_feat + cell];
                    for (val, &wt) in self.vals.iter_mut().zip(weights.iter()) {
                        *val = val.saturating_add(wt);
                    }
                }
                w &= w - 1;
            }
        }
    }

    #[inline]
    pub fn update_add(&mut self, model: &NnueModel, cells: &[usize], is_friendly: bool) {
        let base_feat = if is_friendly { 0 } else { 600 };
        for &cell in cells {
            if cell < 600 {
                let weights = &model.feature_weights[base_feat + cell];
                for (val, &w) in self.vals.iter_mut().zip(weights.iter()) {
                    *val = val.saturating_add(w);
                }
            }
        }
    }

    #[inline]
    pub fn update_sub(&mut self, model: &NnueModel, cells: &[usize], is_friendly: bool) {
        let base_feat = if is_friendly { 0 } else { 600 };
        for &cell in cells {
            if cell < 600 {
                let weights = &model.feature_weights[base_feat + cell];
                for (val, &w) in self.vals.iter_mut().zip(weights.iter()) {
                    *val = val.saturating_sub(w);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_NNUE_BYTES: &[u8] = include_bytes!("../../public/whale_3_0.nnue");

    #[test]
    fn test_dynamic_nnue_loading_and_forward() {
        assert!(NnueModel::from_bytes(&[0, 1, 2]).is_err());
        assert!(NnueModel::from_bytes(b"BAD!1234").is_err());

        let model = NnueModel::from_bytes(TEST_NNUE_BYTES).expect("Valid WNNU v1 bytes");
        assert_eq!(model.feature_weights.len(), NUM_FEATURES);

        let acc_empty = NnueAccumulator::new(&model);
        let score_empty = model.forward(&acc_empty);
        assert!(score_empty.abs() < 1e-4);

        let mut acc_p1 = NnueAccumulator::new(&model);
        acc_p1.update_add(&model, &[10, 11, 12, 13, 14], true);
        assert!(model.forward(&acc_p1) > 0.0);

        let mut acc_p2 = NnueAccumulator::new(&model);
        acc_p2.update_add(&model, &[10, 11, 12, 13, 14], false);
        assert!(model.forward(&acc_p2) < 0.0);
    }
}
