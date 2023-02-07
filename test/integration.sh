#!/bin/bash

set -x

cargo build --release || exit 1

./target/release/devicetree-tool --in-type dts --in-file test/integration_01.dts --out-type dtb --out-file temp.dtb || exit 1

dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1

rm -f temp.dts temp.dtb

./target/release/devicetree-tool --in-type dts --in-file test/integration_02.dts --out-type dtb --out-file temp.dtb || exit 1

dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1

cat temp.dts

rm -f temp.dts temp.dtb

