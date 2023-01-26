mod visualizer;
mod strategy_reader;
mod reader_testing;


fn main() {
    if let Err(e) = visualizer::render_plot() {
        println!("{e}");
    }

    println!("Success");
}
