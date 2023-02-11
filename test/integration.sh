#!/bin/bash

set -x

cargo build --release || exit 1

# case 1
./target/release/devicetree-tool --in-type dts --in-file test/integration_01.dts --out-type dtb --out-file temp.dtb || exit 1

dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1

diff temp.dts test/integration_01.dts || exit 1

rm -f temp.dts temp.dtb

# case 2
./target/release/devicetree-tool --in-type dts --in-file test/integration_02.dts --out-type dtb --out-file temp.dtb || exit 1

dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1

diff temp.dts test/integration_02.dts || exit 1

rm -f temp.dts temp.dtb

# case 3
./target/release/devicetree-tool --in-type dts --in-file test/integration_03.dts --out-type dtb --out-file temp.dtb || exit 1

dtc -I dtb -O dts -o temp.dts temp.dtb || exit 1

diff temp.dts test/integration_03.dts || exit 1

rm -f temp.dts temp.dtb
