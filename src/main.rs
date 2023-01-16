mod fo_visualizer;

fn main() {
    if let Err(e) = fo_visualizer::render_plot(){
        println!("{}",e);
    }
}

//main code can go here and each implementation of visualizers (something that creates a pdf or a page)
//and strategies (simply doing some interaction with markets) can go on our own folders
//this way everyone can manage his own code as preferred and we use the common part to present all the work
