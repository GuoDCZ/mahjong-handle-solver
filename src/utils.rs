use riichi::tile::Tile;
use riichi::tile_group::{KokushiGroup, KoutsuGroup, ShuntsuGroup, TileGroup, ToitsuGroup};

pub const STYLE: &str = "[{elapsed_precise}]-[{eta_precise}] {bar:40} {pos}/{len} {msg}";

pub fn next_tile(tile: Tile) -> Option<Tile> {
    use Tile::*;
    let tile = match tile {
        _1m => _2m,
        _2m => _3m,
        _3m => _4m,
        _4m => _5m,
        _5m => _6m,
        _6m => _7m,
        _7m => _8m,
        _8m => _9m,
        _9m => _1p,
        _1p => _2p,
        _2p => _3p,
        _3p => _4p,
        _4p => _5p,
        _5p => _6p,
        _6p => _7p,
        _7p => _8p,
        _8p => _9p,
        _9p => _1s,
        _1s => _2s,
        _2s => _3s,
        _3s => _4s,
        _4s => _5s,
        _5s => _6s,
        _6s => _7s,
        _7s => _8s,
        _8s => _9s,
        _9s => _1z,
        _1z => _2z,
        _2z => _3z,
        _3z => _4z,
        _4z => _5z,
        _5z => _6z,
        _6z => _7z,
        _7z => return None,
    };
    Some(tile)
}

pub fn toitsu_of_tile(tile: Tile) -> TileGroup {
    use Tile::*;
    use ToitsuGroup::*;
    let toitsu = match tile {
        _1m => _11m,
        _2m => _22m,
        _3m => _33m,
        _4m => _44m,
        _5m => _55m,
        _6m => _66m,
        _7m => _77m,
        _8m => _88m,
        _9m => _99m,
        _1p => _11p,
        _2p => _22p,
        _3p => _33p,
        _4p => _44p,
        _5p => _55p,
        _6p => _66p,
        _7p => _77p,
        _8p => _88p,
        _9p => _99p,
        _1s => _11s,
        _2s => _22s,
        _3s => _33s,
        _4s => _44s,
        _5s => _55s,
        _6s => _66s,
        _7s => _77s,
        _8s => _88s,
        _9s => _99s,
        _1z => _11z,
        _2z => _22z,
        _3z => _33z,
        _4z => _44z,
        _5z => _55z,
        _6z => _66z,
        _7z => _77z,
    };
    TileGroup::Toitsu(toitsu)
}

pub fn tile_of_toitsu(toitsu: ToitsuGroup) -> Tile {
    use Tile::*;
    use ToitsuGroup::*;
    match toitsu {
        _11m => _1m,
        _22m => _2m,
        _33m => _3m,
        _44m => _4m,
        _55m => _5m,
        _66m => _6m,
        _77m => _7m,
        _88m => _8m,
        _99m => _9m,
        _11p => _1p,
        _22p => _2p,
        _33p => _3p,
        _44p => _4p,
        _55p => _5p,
        _66p => _6p,
        _77p => _7p,
        _88p => _8p,
        _99p => _9p,
        _11s => _1s,
        _22s => _2s,
        _33s => _3s,
        _44s => _4s,
        _55s => _5s,
        _66s => _6s,
        _77s => _7s,
        _88s => _8s,
        _99s => _9s,
        _11z => _1z,
        _22z => _2z,
        _33z => _3z,
        _44z => _4z,
        _55z => _5z,
        _66z => _6z,
        _77z => _7z,
    }
}

pub fn koutsu_of_tile(tile: Tile) -> TileGroup {
    use KoutsuGroup::*;
    use Tile::*;
    let koutsu = match tile {
        _1m => _111m,
        _2m => _222m,
        _3m => _333m,
        _4m => _444m,
        _5m => _555m,
        _6m => _666m,
        _7m => _777m,
        _8m => _888m,
        _9m => _999m,
        _1p => _111p,
        _2p => _222p,
        _3p => _333p,
        _4p => _444p,
        _5p => _555p,
        _6p => _666p,
        _7p => _777p,
        _8p => _888p,
        _9p => _999p,
        _1s => _111s,
        _2s => _222s,
        _3s => _333s,
        _4s => _444s,
        _5s => _555s,
        _6s => _666s,
        _7s => _777s,
        _8s => _888s,
        _9s => _999s,
        _1z => _111z,
        _2z => _222z,
        _3z => _333z,
        _4z => _444z,
        _5z => _555z,
        _6z => _666z,
        _7z => _777z,
    };
    TileGroup::Koutsu(koutsu)
}

pub fn shuntsu_of_tile(tile: Tile) -> Option<TileGroup> {
    use ShuntsuGroup::*;
    use Tile::*;
    let shuntsu = match tile {
        _1m => _123m,
        _2m => _234m,
        _3m => _345m,
        _4m => _456m,
        _5m => _567m,
        _6m => _678m,
        _7m => _789m,
        _1p => _123p,
        _2p => _234p,
        _3p => _345p,
        _4p => _456p,
        _5p => _567p,
        _6p => _678p,
        _7p => _789p,
        _1s => _123s,
        _2s => _234s,
        _3s => _345s,
        _4s => _456s,
        _5s => _567s,
        _6s => _678s,
        _7s => _789s,
        _ => return None,
    };
    Some(TileGroup::Shuntsu(shuntsu))
}

pub fn tile_of_kokushi(kokushi: KokushiGroup) -> Tile {
    use KokushiGroup::*;
    use Tile::*;
    match kokushi {
        _119m19p19s1234567z => _1m,
        _199m19p19s1234567z => _9m,
        _19m119p19s1234567z => _1p,
        _19m199p19s1234567z => _9p,
        _19m19p119s1234567z => _1s,
        _19m19p199s1234567z => _9s,
        _19m19p19s11234567z => _1z,
        _19m19p19s12234567z => _2z,
        _19m19p19s12334567z => _3z,
        _19m19p19s12344567z => _4z,
        _19m19p19s12345567z => _5z,
        _19m19p19s12345667z => _6z,
        _19m19p19s12345677z => _7z,
    }
}
