use std::error::Error;
use druid::{AppLauncher, Widget, WindowDesc};
use druid::widget::{Flex, Scroll, SizedBox};
use plotters::prelude::*;
use plotters::prelude::full_palette::{CYAN_900};
use plotters::style::full_palette::{INDIGO_100, INDIGO_400, LIGHTBLUE_600, PURPLE_600, RED_500};
use plotters_druid::Plot;

use super::strategy_reader;

struct Strategy {
    transaction_summary: Vec<(f64, f64)>,
    balance : Vec<(f64,f64,f64,f64)>,
    max_achieved : f64,
    executed_ops : f64,
}
impl Strategy {
    fn new(filename : &str)-> Self{
        let mut transaction_summary = strategy_reader::read(filename)
            .unwrap()
            .into_iter()
            .map(|x|(x.get_day(),x.get_eur()+x.get_usd()+x.get_yen()+x.get_yuan()))
            .collect::<Vec<(f64,f64)>>();
        //By sorting we guarantee a linear shape
        transaction_summary.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let balance = strategy_reader::read(filename)
            .unwrap()
            .into_iter()
            .map(|x|(x.get_eur(),x.get_usd(),x.get_yen(),x.get_yuan()))
            .collect::<Vec<(f64,f64,f64,f64)>>();
        let max_achieved = transaction_summary.iter().max_by(|x, y| x.1.partial_cmp(&y.1).unwrap()).expect("No acceptable values found").1;
        let executed_ops = transaction_summary[transaction_summary.len()-1].0;
        Strategy {
            transaction_summary,
            balance ,
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
        .window_size((1600.0, 900.0))
        .resizable(true);

    AppLauncher::with_window(main_window)
        .launch(())
        .expect("launch failed");
    Ok(())
}

fn chart_builder() -> impl Widget<()>{
    let trades = load_strategies();
    let selected = 0;
    let mut row = Flex::row()
        .with_child(SizedBox::new(Plot::new(move |_size, _data, root| {

            root.fill(&WHITE).unwrap();
            //The chart will be put on the window
            let mut chart = ChartBuilder::on(root);

            //Shifting the chart away from the window borders
            chart
                .margin(20)
                .set_left_and_bottom_label_area_size(45);

            let observed = &trades[selected];
            //I need this to draw inside the area of the chart
            let mut chart_context = chart.build_cartesian_2d(0.0..(observed.executed_ops+0.5), 0.0..(observed.max_achieved *1.25)).unwrap();
            //Background grid
            chart_context.configure_mesh().draw().unwrap();

            //base of the plot (eur)
            let observed_eur = observed
                .transaction_summary
                .iter()
                .zip(observed.balance.iter())
                .map(|((day,_),(eur,_,_,_)),| (*day,*eur))
                .collect::<Vec<(f64,f64)>>();
            //usd on top of eur
            let observed_usd = observed
                .transaction_summary
                .iter()
                .zip(observed.balance.iter())
                .map(|((day,_),(_,usd,_,_)),| (day,usd))
                .zip(observed_eur.iter())
                .map(|((day,usd),(_,eur)),| (*day,*eur+*usd))
                .collect::<Vec<(f64,f64)>>();
            //yen on top of usd
            let observed_yen = observed
                .transaction_summary
                .iter()
                .zip(observed.balance.iter())
                .map(|((day,_),(_,_,yen,_)),| (day,yen))
                .zip(observed_usd.iter())
                .map(|((day,yen),(_,usd)),| (*day,*usd+*yen))
                .collect::<Vec<(f64,f64)>>();
            //yuan on top of yen
            let observed_yuan = observed
                .transaction_summary
                .iter()
                .zip(observed.balance.iter())
                .map(|((day,_),(_,_,_,yuan)),| (*day,*yuan))
                .zip(observed_yen.iter())
                .map(|((day,yuan),(_,yen)),| (day,*yen+yuan))
                .collect::<Vec<(f64,f64)>>();

            //to draw a line connecting all the values of eur
            chart_context.draw_series(AreaSeries::new(
                observed_eur.clone(),
                0.0,
                INDIGO_100.mix(0.33),
            ).border_style(INDIGO_400)).unwrap();
            chart_context.draw_series(AreaSeries::new(
                observed_usd.clone(),
                0.0,
                INDIGO_100.mix(0.33),
            ).border_style(RED_500)).unwrap();
            chart_context.draw_series(AreaSeries::new(
                observed_yen.clone(),
                0.0,
                INDIGO_100.mix(0.33),
            ).border_style(LIGHTBLUE_600)).unwrap();

            chart_context.draw_series(AreaSeries::new(
                observed_yuan.clone(),
                0.0,
                INDIGO_100.mix(0.33),
            ).border_style(PURPLE_600)).unwrap();

            //to have a labelled point on every value update on the plot
            chart_context.draw_series(PointSeries::of_element(
                observed_eur.clone(),
                3,
                &CYAN_900,
                &|c, s, st| {
                    EmptyElement::at(c)    // Composed element on-the-fly
                        + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

                },
            )).unwrap();

            chart_context.draw_series(PointSeries::of_element(
                observed_usd.clone(),
                3,
                &CYAN_900,
                &|c, s, st| {
                    EmptyElement::at(c)    // Composed element on-the-fly
                        + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

                },
            )).unwrap();

            chart_context.draw_series(PointSeries::of_element(
                observed_yen.clone(),
                3,
                &CYAN_900,
                &|c, s, st| {
                    EmptyElement::at(c)    // Composed element on-the-fly
                        + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

                },
            )).unwrap();

            chart_context.draw_series(PointSeries::of_element(
                observed_yuan.clone(),
                3,
                &CYAN_900,
                &|c, s, st| {
                    EmptyElement::at(c)    // Composed element on-the-fly
                        + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point
                        + Text::new(format!("€ {:.2}", c.1), (-2,-30), ("sans-serif", 14).into_font()) //Every point will have its total value labelled
                },
            )).unwrap();


        })).width(800.0).height(700.0));

    let trades = load_strategies(); //recreated because of move from the previous plot
    let selected = 0;
    row.add_spacer(10.0);
    row.add_child(SizedBox::new(Plot::new(move |_size, _data, root| {

        root.fill(&WHITE).unwrap();
        //The chart will be put on the window
        let mut chart = ChartBuilder::on(root);

        //Shifting the chart away from the window borders
        chart
            .margin(20)
            .set_left_and_bottom_label_area_size(45);

        let observed = &trades[selected];
        //I need this to draw inside the area of the chart
        let mut chart_context = chart.build_cartesian_2d(0.0..(observed.executed_ops+0.5), 0.0..(observed.max_achieved *1.25)).unwrap();
        //Background grid
        chart_context.configure_mesh().draw().unwrap();

        //base of the plot (eur)
        let observed_eur = observed
            .transaction_summary
            .iter()
            .zip(observed.balance.iter())
            .map(|((day,_),(eur,_,_,_)),| (*day,*eur))
            .collect::<Vec<(f64,f64)>>();
        //usd on top of eur
        let observed_usd = observed
            .transaction_summary
            .iter()
            .zip(observed.balance.iter())
            .map(|((day,_),(_,usd,_,_)),| (day,usd))
            .zip(observed_eur.iter())
            .map(|((day,usd),(_,eur)),| (*day,*eur+*usd))
            .collect::<Vec<(f64,f64)>>();
        //yen on top of usd
        let observed_yen = observed
            .transaction_summary
            .iter()
            .zip(observed.balance.iter())
            .map(|((day,_),(_,_,yen,_)),| (day,yen))
            .zip(observed_usd.iter())
            .map(|((day,yen),(_,usd)),| (*day,*usd+*yen))
            .collect::<Vec<(f64,f64)>>();
        //yuan on top of yen
        let observed_yuan = observed
            .transaction_summary
            .iter()
            .zip(observed.balance.iter())
            .map(|((day,_),(_,_,_,yuan)),| (*day,*yuan))
            .zip(observed_yen.iter())
            .map(|((day,yuan),(_,yen)),| (day,*yen+yuan))
            .collect::<Vec<(f64,f64)>>();

        //to draw a line connecting all the values of eur
        chart_context.draw_series(AreaSeries::new(
            observed_eur.clone(),
            0.0,
            INDIGO_100.mix(0.33),
        ).border_style(INDIGO_400)).unwrap();
        chart_context.draw_series(AreaSeries::new(
            observed_usd.clone(),
            0.0,
            INDIGO_100.mix(0.33),
        ).border_style(RED_500)).unwrap();
        chart_context.draw_series(AreaSeries::new(
            observed_yen.clone(),
            0.0,
            INDIGO_100.mix(0.33),
        ).border_style(LIGHTBLUE_600)).unwrap();

        chart_context.draw_series(AreaSeries::new(
            observed_yuan.clone(),
            0.0,
            INDIGO_100.mix(0.33),
        ).border_style(PURPLE_600)).unwrap();

        //to have a labelled point on every value update on the plot
        chart_context.draw_series(PointSeries::of_element(
            observed_eur.clone(),
            3,
            &CYAN_900,
            &|c, s, st| {
                EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

            },
        )).unwrap();

        chart_context.draw_series(PointSeries::of_element(
            observed_usd.clone(),
            3,
            &CYAN_900,
            &|c, s, st| {
                EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

            },
        )).unwrap();

        chart_context.draw_series(PointSeries::of_element(
            observed_yen.clone(),
            3,
            &CYAN_900,
            &|c, s, st| {
                EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point

            },
        )).unwrap();

        chart_context.draw_series(PointSeries::of_element(
            observed_yuan.clone(),
            3,
            &CYAN_900,
            &|c, s, st| {
                EmptyElement::at(c)    // Composed element on-the-fly
                    + Circle::new((0,0),s,st.filled()) // New pixel coordinate is established in c, with a filled circle on the point
                    + Text::new(format!("€ {:.2}", c.1), (-2,-30), ("sans-serif", 14).into_font()) //Every point will have its total value labelled
            },
        )).unwrap();


    })).width(800.0).height(700.0));

    return row;




}