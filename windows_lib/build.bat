rustup run stable-i686-pc-windows-gnu cargo build --release

del "C:\Users\Dan\Documents\GameMaker\Projects\LD49_8.gmx\extensions\Extension2\world_generators.dll"
copy "C:\Users\Dan\world_generators\target\release\world_generators.dll" "C:\Users\Dan\Documents\GameMaker\Projects\LD49_8.gmx\extensions\Extension2\"