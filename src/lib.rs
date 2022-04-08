pub mod receipt_read {
    /// Reads receipts and sort their items in a vector of Item structures composed of name and
    /// price.
    ///
    /// TODO: try to sort trash data (ex: unwanted lines from Real tickets)
    
    use leptess::LepTess;
    use std::{env, f32::INFINITY};

    #[derive(Debug)]
    pub struct Item {
        item_name: String,
        item_price: f32,
    }

    pub fn sort_lines(file_name: &str) -> Vec<String> {
        /// Read each lines on the receipt and only keeps the ones that are supposed to be
        /// interesting, namely those with numbers between a start (shop dependant) and an end
        /// (usually "Summe").

        let prefix = env::var("TESSDATA_PREFIX").unwrap();
        // TESSDATA_PREFIX is environement variable that needs to be configured.
        // non english training data here: https://github.com/tesseract-ocr/tessdata
        
        let mut lt = LepTess::new(Some(&prefix), "deu").unwrap();
        lt.set_image(file_name).unwrap();

        let tmp_text = lt.get_utf8_text().unwrap().to_lowercase().to_owned();

        let lines: Vec<&str> = tmp_text.split("\n").collect();
        let mut kept_lines: Vec<String> = Vec::new();

        let mut keeping: bool = false;
        let mut _first_char: bool;
        let mut prev_line: &str = "";
        let mut karten_or_geld: bool = false;
        for line in &lines {
            if line.contains("sie sparen") || line.contains("/") || line.contains("abatt") {
                continue;
            }

            if line.contains("eur") || line.contains("fur") || line.contains("real") {
                keeping = true;
            }

            if line.contains("sum") {
                kept_lines.push(line.to_string());
                break;
            }
            if line.contains("kartenzahlung") {
                kept_lines.push(line.to_string());
                if !karten_or_geld {
                    karten_or_geld = true;
                    continue;
                } else {
                    break;
                }
            }
            if line.contains("rückgeld") {
                kept_lines.push(line.to_string());
                if !karten_or_geld {
                    karten_or_geld = true;
                    continue;
                } else {
                    break;
                }
            }

            if keeping {
                _first_char = true;
                for chars in line.clone().chars() {
                    if chars.is_numeric() {
                        if _first_char {
                            kept_lines.push(prev_line.to_string());
                            _first_char = false;
                        }
                        kept_lines.push(line.to_string());
                        break;
                    }
                    _first_char = false;
                }
                prev_line = line.clone();
            }
        }
        kept_lines
    }

    pub fn sort_items(lines: Vec<String>) -> Vec<Item> {
        /// Read the lines kept with the previous function and use them to keep items by name and
        /// price, turning them into Item objects and returning a vector of said objects
        /// (basically, the ticket)
        /// TODO: manage the cases where duplicate items are counted before their names instead of
        /// after

        let mut items: Vec<Item> = Vec::new();
        let mut tmp_name: String = String::from("");
        let mut tmp_value: f32 = INFINITY;
        for line in lines {
            let work_line: Vec<&str> = line.split(" ").collect();
            if tmp_value != INFINITY && tmp_name.ne("") {
                tmp_name = String::from("");
                tmp_value = INFINITY;
            }
           /* else if tmp_value != INFINITY && tmp_name == String::from(""){
                tmp_value = INFINITY;
            }*/
            //tmp_value = INFINITY;
            'words: for word in &work_line {
                for c in word.chars() {
                    if c.is_alphabetic() {
                        tmp_name += (word.to_owned().to_owned() + " ").as_str();
                        break;
                    }
                    if c.is_numeric() {
                        if work_line[work_line.len() - 2]
                            .chars()
                            .nth(0)
                            .unwrap()
                            .is_numeric()
                        {
                            tmp_value = work_line[work_line.len() - 2]
                                .replace(",", ".")
                                .parse::<f32>()
                                .unwrap();
                            if tmp_value != INFINITY && tmp_name.ne(""){
                            items.push(Item {
                                item_name: tmp_name.clone(),
                                item_price: tmp_value,
                            });
                            }
                            break 'words;
                        } else if work_line[work_line.len() - 1]
                            .chars()
                            .nth(0)
                            .unwrap()
                            .is_numeric()
                        {
                           // println!("line: {:?}", work_line);
                            tmp_value = work_line[work_line.len() - 1]
                                .replace(",", ".")
                                .parse::<f32>()
                                .unwrap();
                            if tmp_value != INFINITY && tmp_name.ne(""){
                            items.push(Item {
                                item_name: tmp_name.clone(),
                                item_price: tmp_value,
                            });
                            }
                            break 'words;
                        }
                    }
                }
            }
        }

        let mut sum: bool = false;
        tmp_value = 0.0;
        for i in &items {
            if i.item_name.contains("summe") {
                sum = true;
                break;
            }

            if i.item_name.contains("kartenzahlung") && !sum {
                tmp_value += i.item_price;
            }

            if i.item_name.contains("rückgeld") && !sum {
                tmp_value += i.item_price;
            }
        }

        if !sum {
            items.push(Item {
                item_name: String::from("summe"),
                item_price: tmp_value,
            });
        }

        items
    }
}
