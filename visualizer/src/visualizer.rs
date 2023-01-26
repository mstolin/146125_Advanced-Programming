use std::error::Error;
use druid::{AppLauncher, Widget, WindowDesc};
use plotters::prelude::*;
use plotters::prelude::full_palette::CYAN_900;
use plotters::style::full_palette::INDIGO_400;
use plotters_druid::Plot;

use super::strategy_reader;

struct Strategy {
    transactions : Vec<(f64,f64)>,
    max_achieved : f64,
    executed_ops : f64,
}
impl Strategy {
    fn new(filename : &str)-> Self{
        let mut transactions = strategy_reader::read(filename)
            .unwrap()
            .into_iter()
            .map(|x|(x.get_id(),x.get_possession()))
            .collect::<Vec<(f64,f64)>>();
        //By sorting we guarantee a linear shape
        transactions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let max_achieved = transactions.iter().max_by(|x, y| x.1.partial_cmp(&y.1).unwrap()).expect("No acceptable values found").1;
        let executed_ops = transactions[transactions.len()-1].0;
        Strategy {
            transactions ,
            max_achieved ,
            executed_ops

        }
    }
}
fn load_strategies() -> Vec<Strategy>{
    let mut trades = vec![];
    let files_found = strategy_reader::find_all_available();
    for file in files_found{
        trades.push(Strategy::new(&file));
    }
    trades
}

pub fn render_plot()-> Result<(),Box<dyn Error>>{
    let main_window = WindowDesc::new(chart_builder)
        .title("Strategy result")
        .window_size((1280.0, 900.0))
        .resizable(true);

    AppLauncher::with_window(main_window)
        .launch(())
        .expect("launch failed");
    Ok(())
}

fn chart_builder() -> impl Widget<()>{
    let trades = load_strategies();
    let selected = 6%trades.len();

    Plot::new(move |_size, _data, root| {

        root.fill(&WHITE).unwrap();
        //The chart will be put on the window
        let mut chart = ChartBuilder::on(root);

        //Shifting the chart away from the window borders
        chart
            .margin(20)
            .set_left_and_bottom_label_area_size(45);

        //I need this to draw inside the area of the chart
        let mut chart_context = chart.build_cartesian_2d(0.0..(trades[selected].executed_ops), 0.0..(trades[selected].max_achieved *1.25)).unwrap();

        //Background grid
        chart_context.configure_mesh().draw().unwrap();

        //to draw a line connecting all the values
        chart_context.draw_series(AreaSeries::new(
            trades[selected].transactions.clone(),
            0.0,
            CYAN_900.mix(0.33),
        ).border_style(INDIGO_400)).unwrap();

        //to have a labelled point on every value update on the plot
        chart_context.draw_series(PointSeries::of_element(
            trades[selected].transactions.clone(),
            5,
            &CYAN_900,
            &|c, s, st| {
                EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point
                    + Text::new(format!("{:?}", c.1), (-10, -20), ("sans-serif", 14).into_font()) //Every point will have its value labelled
            },
        )).unwrap();

    })

}
