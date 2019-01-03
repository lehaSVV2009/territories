import findClosedLoops from "./findClosedLoops";

describe("findClosedLoops", () => {
  it("should find closed loop as a single cell in matrix", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 1],
      [1, 1, 0, 1, 1],
      [1, 1, 1, 1, 1],
      [1, 1, 1, 1, 1]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(1);
    expect(loops[0].cells).toHaveLength(1);
    expect(loops[0].cells[0]).toEqual({ rowIndex: 1, columnIndex: 2 });
  });

  it("should not find any loops", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 1],
      [1, 1, 1, 1, 1],
      [1, 1, 1, 1, 1],
      [1, 1, 1, 1, 1]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(0);
  });

  it("should find closed loop as a 3 cells in matrix", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 1],
      [1, 1, 0, 0, 1],
      [1, 1, 0, 1, 1],
      [1, 1, 1, 1, 1]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(1);
    expect(loops[0].cells).toHaveLength(3);
    expect(loops[0].cells[0]).toEqual({ rowIndex: 1, columnIndex: 2 });
    expect(loops[0].cells[1]).toEqual({ rowIndex: 2, columnIndex: 2 });
    expect(loops[0].cells[2]).toEqual({ rowIndex: 1, columnIndex: 3 });
  });

  it("should find 2 closed loops with matrix border in matrix", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 0],
      [1, 0, 1, 0, 0],
      [0, 0, 1, 0, 1],
      [1, 1, 1, 1, 1]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(2);
    expect(loops[0].cells).toHaveLength(4);
    expect(loops[1].cells).toHaveLength(3);
  });

  it("should find closed loop with matrix border in matrix", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 1],
      [1, 0, 0, 1, 0],
      [1, 1, 0, 1, 0],
      [1, 0, 0, 0, 1]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(2);
    expect(loops[0].cells).toHaveLength(6);
    expect(loops[1].cells).toHaveLength(2);
  });

  it("should find closed loop with neigbours values", () => {
    // given
    const matrix = [
      [1, 2, 3, 4, 5],
      [6, 7, 0, 0, 10],
      [11, 12, 13, 14, 15],
      [16, 17, 18, 19, 20]
    ];
    const loopHabitantsFilter = ({ value }) => value === 0;

    // when
    const loops = findClosedLoops({ matrix, loopHabitantsFilter });

    // then
    expect(loops).toHaveLength(1);
    expect(loops[0].neigboursValues).toContain(3);
    expect(loops[0].neigboursValues).toContain(4);
    expect(loops[0].neigboursValues).toContain(7);
    expect(loops[0].neigboursValues).toContain(10);
    expect(loops[0].neigboursValues).toContain(13);
    expect(loops[0].neigboursValues).toContain(14);
  });

  it("should mark entire matrix as closed loop if loop filter is missing", () => {
    // given
    const matrix = [
      [1, 1, 1, 1, 1],
      [1, 1, 0, 1, 1],
      [1, 1, 1, 1, 1],
      [1, 1, 1, 1, 1]
    ];

    // when
    const loops = findClosedLoops({ matrix });

    // then
    expect(loops).toHaveLength(1);
    expect(loops[0].cells).toHaveLength(20);
  });
});
