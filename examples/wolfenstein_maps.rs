use bevystein;

fn main() {
    let maps = bevystein::map_parser::load_maps(
        "shareware/MAPHEAD.WL1",
        "shareware/GAMEMAPS.WL1",
        Some(1),
    );

    println!("Wolfenstein map 1 layout:");
    println!("{}", maps[0]);
}
