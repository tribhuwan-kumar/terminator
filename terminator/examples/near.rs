use terminator::{AutomationError, Desktop};

#[tokio::main]
async fn main() -> Result<(), AutomationError> {
    tracing_subscriber::fmt::init();

    let desktop = Desktop::new(true, false)?;

    let opened_app = desktop.focused_element()?;

    // plan-info automation id
    let plan_info = opened_app
        .locator("AutomationId:plan-info")?
        .first(None)
        .await?;
    let plan_info_text = plan_info.text(100)?;
    println!("Plan info text: {plan_info_text}");

    // let tree = opened_app.to_serializable_tree(5);
    // let flat_tree = tree.children.iter().flatten().collect::<Vec<_>>();
    // // println!("{}", serde_json::to_string_pretty(&tree).unwrap());

    // // get the inde of first element which is a text element containing $ with a group before and after
    // let mut start: i32 = -1;

    // // get the index of the first element which is a text element containing a number followed
    // // by a text element containing "of " followed by a text element containing a number
    // let mut end: i32 = -1;

    // for (i, element) in flat_tree.iter().enumerate() {
    //     if element.role == "Text" {
    //         let text = element.text.clone().unwrap_or_default();
    //         if text.contains("$")
    //             && i > 0
    //             && i + 1 < flat_tree.len()
    //             && flat_tree[i - 1].role == "Group"
    //             && flat_tree[i + 1].role == "Group"
    //         {
    //             println!("Found text element containing $: {}", text);
    //             start = i as i32;
    //         }
    //         if text.contains("of ")
    //             && i > 0
    //             && i + 1 < flat_tree.len()
    //             && flat_tree[i - 1].role == "Group"
    //             && flat_tree[i + 1].role == "Group"
    //         {
    //             println!("Found text element containing of: {}", text);
    //             end = i as i32;
    //         }
    //     }
    // }

    // // print the concatenated text of the elements between start and end
    // if start != -1 && end != -1 {
    //     let text = flat_tree[start as usize..end as usize]
    //         .iter()
    //         .map(|e| e.text.clone().unwrap_or_default())
    //         .collect::<Vec<_>>()
    //         .join("");
    //     println!("Concatenated text: {}", text);
    // }

    Ok(())
}
