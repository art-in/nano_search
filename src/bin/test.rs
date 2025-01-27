use nano_search::utils::Print;

fn main() {
    let output =
        std::fs::File::create("test_results").expect("file should be created");
    let mut output = std::io::BufWriter::new(output);

    let queries = nano_search::data::query::get_queries();
    let index = nano_search::fulltext::index::build_index();
    let stop_words = nano_search::data::stop_words::parse_stop_words();

    let mut precisions = Vec::new();
    let mut recalls = Vec::new();

    output.print("---");

    for query in queries {
        output.println("---");
        output.println(format!("query id: {}", query.id));
        let found_docids = nano_search::fulltext::search::search(
            &query.text,
            &index,
            &stop_words,
        );

        let mut precise_docids: u64 = 0;

        output.print(format!(
            "expected docids (size={}):",
            query.expected_docids.len()
        ));
        for expected_docid in &query.expected_docids {
            output.print(format!(" {expected_docid}"));
        }
        output.println("");

        output.print(format!("found docids (size={}):", found_docids.len()));
        for found_docid in &found_docids {
            output.print(format!(" {found_docid}"));
            if query.expected_docids.contains(found_docid) {
                precise_docids += 1;
            }
        }
        output.println("");

        output.println(format!("precise docids: {}", precise_docids));

        let precision = if found_docids.is_empty() {
            if query.expected_docids.is_empty() {
                1_f64
            } else {
                0_f64
            }
        } else {
            precise_docids as f64 / found_docids.len() as f64
        };

        let recall = if query.expected_docids.is_empty() {
            1_f64
        } else {
            precise_docids as f64 / query.expected_docids.len() as f64
        };

        precisions.push(precision);
        recalls.push(recall);

        output.println(format!("precision: {}", precision));
        output.println(format_args!("recall: {}", recall));
    }
    output.println("");

    let average_precision =
        precisions.iter().sum::<f64>() / precisions.len() as f64;
    let average_recall = recalls.iter().sum::<f64>() / recalls.len() as f64;

    output.println("===");
    let average_precision_message =
        format!("average precision: {}", average_precision);
    let average_recall_message = format!("average recall: {}", average_recall);
    output.println(&average_precision_message);
    output.println(&average_recall_message);

    println!("{average_precision_message}");
    println!("{average_recall_message}");
}
