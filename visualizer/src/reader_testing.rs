
#[cfg(test)]
mod reader_tests{
    use crate::strategy_reader::{read,save};

    #[test]
    fn save_and_read(){
        let ops = vec![(0, 10.0),(3, 42.1),(2, 37.4)];
        save(&ops,"save_test");
        let expected_content = read("save_test.json");
        println!("The file contains :\n{:?}",expected_content.as_ref().unwrap());
        assert_eq!(expected_content.is_ok(),true);
    }

}