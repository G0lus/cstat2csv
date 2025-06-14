use clap::Parser;
use scraper::{self};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the summary .html file.
    /// Output file will be created in the same directory with the same name as the summary file.
    #[arg(short, long)]
    path: String,
    /// Path to output file.
    #[arg(short, long)]
    output: String,
}

#[derive(Serialize, Debug)]
struct ReportEntry {
    file: String,
    line: usize,
    tag: String,
    severity: String,
    message: String,
}

fn parse_file(file: &std::path::Path) -> Result<Vec<ReportEntry>, std::io::Error> {
    let text = std::fs::read_to_string(file)?;
    let html = scraper::Html::parse_document(&text.as_str());
    let table_select = scraper::Selector::parse("table").unwrap();
    let rows_select = scraper::Selector::parse("tr").unwrap();
    let rows_data_select = scraper::Selector::parse("td").unwrap();
    let table = html.select(&table_select).next().unwrap();

    let rows = table.select(&rows_select).skip(2);
    println!(
        "Parsing {}: {} messages",
        file.to_str().unwrap(),
        rows.clone().into_iter().count(),
    );

    let mut vec = Vec::<ReportEntry>::new();
    for row in rows {
        let data = row.select(&rows_data_select);
        let elems = data.map(|elem| elem.inner_html()).collect::<Vec<String>>();
        if elems.len() > 4 {
            let entry = ReportEntry {
                file: elems.iter().nth(0).unwrap().to_string(),
                line: elems.iter().nth(1).unwrap().parse::<usize>().unwrap(),
                tag: elems.iter().nth(2).unwrap().to_string(),
                severity: elems.iter().nth(3).unwrap().to_string(),
                message: elems.iter().nth(4).unwrap().to_string(),
            };
            vec.push(entry);
        }
    }
    return Ok(vec);
}

fn get_files_list(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
    let source = std::fs::read_to_string(path)?;

    let html_source = scraper::html::Html::parse_document(source.as_str());

    let html_table = scraper::selector::Selector::parse("table").unwrap();
    let tables = html_source.select(&html_table);
    if tables.clone().count() == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error parsing file",
        ));
    }
    let hyperlinks = tables.into_iter().find(|element| {
        element
            .attr("id")
            .is_some_and(|id| id.contains("hyperlink-info"))
    });
    if hyperlinks.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error parsing file",
        ));
    }
    let link_selector = scraper::Selector::parse("a").unwrap();
    let filenames =
        hyperlinks
            .unwrap()
            .select(&link_selector)
            .fold(Vec::<String>::new(), |mut names, name| {
                names.push(name.value().attr("href").unwrap().to_string());
                names
            });
    Ok(filenames)
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let path = std::path::Path::new(args.path.as_str());
    let dir = path.parent().unwrap();

    let mut out_writer = csv::Writer::from_path(args.output)?;

    let files = get_files_list(path)?;

    for file in files {
        let path = format_args!("{}/{}", dir.to_str().unwrap(), file.as_str()).to_string();
        let status = parse_file(std::path::Path::new(path.as_str()));
        let _ = status.as_ref().and_then(|report| {
            for entry in report {
                let _ = out_writer.serialize(entry);
            }
            Ok(())
        });
    }
    out_writer.flush()?;
    Ok(())
}

#[test]
fn test_parsing() -> Result<(), std::io::Error> {
    let source = std::path::Path::new("Report/Boot/BLLm_bootMain.c.html");

    parse_file(source)?;
    return Ok(());
}

#[test]
fn test_getting_file_list() -> Result<(), std::io::Error> {
    let source = std::path::Path::new("Report/Boot/Boot.html");

    let files = get_files_list(source);
    if files.is_ok_and(|f| f.len() > 0) {
        return Ok(());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error parsing file",
        ));
    }
}
