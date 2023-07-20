#!/bin/sh
cargo readme | sed -e 's,../images/,images/,g' > ./README.md
