mod fo_visualzer;

fn main() {
    fo_visualzer::render_plot();
}

//main code can go here and each implementation of visualizers (something that creates a pdf or a page)
//and strategies (simply doing some interaction with markets) can go on our own folders
//this way everyone can manage his own code as preferred and we use the common part to present all the work
