#!/bin/bash

set -x

cargo build --release || exit 1

# Convert a DTS file to DTB file with `device-tool`, and convert the DTB file
# back to a DTS file with `dtc`.
# Compare if there is any difference between the original DTS file and the 
# converted-back DTS file.
test_input_dts_output_dtb() {
    # Parameter 1: The path of the original DTS file
    DTS_FILE="$1"
    ./target/release/devicetree-tool --in-type dts --in-file "${DTS_FILE}" \
        --out-type dtb --out-file temp.dtb || exit 1
    dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1
    diff temp.dts "${DTS_FILE}"

    RES=$?
    if [ $RES -ne 0 ]; then
        echo "Converted-back DTS is not same as the original DTS"
        echo "-------------"
        echo "Converted-back DTS"
        cat temp.dts
        echo "-------------"
        echo "Original DTS:"
        cat "${DTS_FILE}"
        echo "-------------"
        exit ${RES}
    fi

    rm -f temp.dts temp.dtb
}

test_input_dts_output_dtb "test/integration_01.dts"
test_input_dts_output_dtb "test/integration_02.dts"
test_input_dts_output_dtb "test/integration_03.dts"
