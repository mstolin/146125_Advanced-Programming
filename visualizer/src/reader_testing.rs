
#[cfg(test)]
mod reader_tests{
    use crate::strategy_reader::{read,save};

    #[test]
    fn save_and_read(){
        let ops = vec![(0, 1000000.0, 0.0, 0.0, 0.0),
                       (1, 171150.75, 20091.201, 0.0, 25114.344),
                       (2, 7891.1123, 20091.201, 0.0, 25114.344),
                       (3, 29669.178, 20091.201, 0.0, 25114.344),
                       (4, 20376.188, 20091.201, 0.0, 25114.344),
                       (5, 11038.449, 20091.201, 0.0, 25114.344),
                       (6, 4425.7246, 20091.201, 0.0, 25114.344),
                       (7, 14692.314, 20091.201, 0.0, 0.0)];

        save(&ops,"save_test");
        println!("The file was saved successfully");
        let expected_content = read("save_test.json");
        println!("The file contains :\n{:?}",expected_content.as_ref().unwrap());
        assert_eq!(expected_content.is_ok(),true);
    }

}