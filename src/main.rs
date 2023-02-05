use utils::timer::Timer;

mod day01_calorie_counting;
mod day02_rock_paper_scissors;
mod day03_rucksack_reorganization;
mod day04_camp_cleanup;
mod day05_supply_stacks;
mod day06_tuning_trouble;
mod day07_no_space_left_on_device;
mod day08_treetop_tree_house;
mod day09_rope_bridge;
mod day10_cathode_ray_tube;
mod day11_monkey_in_the_middle;
mod day12_hill_climbing_algorithm;
mod day13_distress_signal;
mod day14_regolith_reservoir;
mod day15_beacon_exclusion_zone;
mod day16_proboscidea_volcanium;
mod day17_pyroclastic_flow;
mod day18_boiling_boulders;
mod day19_not_enough_minerals;
mod day20_grove_positioning_system;
mod day21_monkey_math;

fn main() {
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms.", elapsed.as_millis()));
    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("argument should be an integer")
    } else {
        21
    };
    println!("running day {}\n", day);
    match day {
        1 => day01_calorie_counting::run(),
        2 => day02_rock_paper_scissors::run(),
        3 => day03_rucksack_reorganization::run(),
        4 => day04_camp_cleanup::run(),
        5 => day05_supply_stacks::run(),
        6 => day06_tuning_trouble::run(),
        7 => day07_no_space_left_on_device::run(),
        8 => day08_treetop_tree_house::run(),
        9 => day09_rope_bridge::run(),
        10 => day10_cathode_ray_tube::run(),
        11 => day11_monkey_in_the_middle::run(),
        12 => day12_hill_climbing_algorithm::run(),
        13 => day13_distress_signal::run(),
        14 => day14_regolith_reservoir::run(),
        15 => day15_beacon_exclusion_zone::run(),
        16 => day16_proboscidea_volcanium::run(),
        17 => day17_pyroclastic_flow::run(),
        18 => day18_boiling_boulders::run(),
        19 => day19_not_enough_minerals::run(),
        20 => day20_grove_positioning_system::run(),
        21 => day21_monkey_math::run(),
        _ => panic!("day {} not found", day),
    }
}