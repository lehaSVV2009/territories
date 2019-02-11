export default input => {
  return {
    best:
      1 /
      (1 +
        1 /
          Math.exp(
            33.09363555908203 -
              (18.816131591796875 * 1) /
                (1 +
                  1 /
                    Math.exp(
                      20.226030349731445 +
                        0.006019400432705879 * input["turn"] +
                        6.525365829467773 * input["columnIndex"] +
                        4.9635748863220215 * input["rowIndex"] +
                        25.573339462280273 * input["rectangleHeight"] -
                        41.287147521972656 * input["rectangleWidth"] -
                        5.127196311950684 * input["emptyCellsCount"]
                    )) -
              (18.491500854492188 * 1) /
                (1 +
                  1 /
                    Math.exp(
                      -13.770045280456543 -
                        0.5661401748657227 * input["turn"] +
                        44.73406982421875 * input["columnIndex"] +
                        5.764463424682617 * input["rowIndex"] -
                        3.1857664585113525 * input["rectangleHeight"] +
                        6.251440525054932 * input["rectangleWidth"] -
                        18.625091552734375 * input["emptyCellsCount"]
                    )) -
              (14.760644912719727 * 1) /
                (1 +
                  1 /
                    Math.exp(
                      2.9342174530029297 -
                        0.06486324220895767 * input["turn"] +
                        18.163259506225586 * input["columnIndex"] -
                        20.946380615234375 * input["rowIndex"] +
                        13.361632347106934 * input["rectangleHeight"] -
                        14.830757141113281 * input["rectangleWidth"] +
                        15.96739387512207 * input["emptyCellsCount"]
                    ))
          ))
  };
};
