pub fn level(serialized: &str) -> String {
    let legend_comment = r#"For every tile character is an index corresponding to an item_id in this legend.
// The mapping is defined as follows:
//   * Digits ('0' through '9') are assigned indices 0 to 9.
//   * Letters ('A', 'B', 'C', 'D', 'E', 'G', 'H', 'K', 'N', 'O', 'P', 'Q', 'R', 'S', 'U', 'Z')
//     are assigned indices 10 to 25.
//   * Symbols (e.g., '@', '#', '$', '%', '&', '*', '+', '=', '^', '~', '?', '!')
//     are assigned indices 26 and above.
//   * The blank symbol ('_') indicates no occupying tile.
//   * Any character not covered will be treated as if blank."#;
    let surface_comment = r#"The `surface:` field holds every tilemap layout as a multiline string:
//   * Each line corresponds to a row in the level grid.
//   * Tiles each row are separated by the pipe character (`|`).
//   * Each tile is represented by a single character defined in the legend.
//   * Blank tiles are denoted by '_' and are omitted from world gen.
//   * Edit this layout to modify the level's structure and tile placement.
//   * Tip: Use a monospace font for readability."#;

    let serialized =
        insert_comment_preserve_indent_no_regex(&serialized, legend_comment, "legend:");

    let serialized =
        insert_comment_preserve_indent_no_regex(&serialized, surface_comment, "surface:");

    let serialized = wrap_surface_entries(&serialized);

    serialized
}
/// Wraps a single line containing a `content:` field by replacing the quoted
/// string with a raw string literal. It converts literal "\n" sequences to actual newlines,
/// so that the first line appears immediately after r#", and subsequent lines are indented with 18 spaces.
pub fn wrap_content_line(line: &str) -> String {
    // Look for the "content:" token.
    if let Some(content_idx) = line.find("content:") {
        // Split the line into the part before "content:" and after.
        let (before, after_content) = line.split_at(content_idx);
        // Now, after_content should start with "content:".
        // Find the first double quote in after_content.
        if let Some(start_quote_idx) = after_content.find('\"') {
            let prefix = &after_content[..start_quote_idx]; // includes "content:" and any spaces before the quote
            let remainder = &after_content[start_quote_idx..]; // starts with the opening quote

            // Find the closing quote.
            if let Some(end_quote_idx) = remainder.rfind('\"') {
                // Determine if there's a trailing comma after the closing quote.
                let (quoted, trailing) = if remainder[end_quote_idx..].starts_with("\",") {
                    (&remainder[1..end_quote_idx], ",")
                } else {
                    (&remainder[1..end_quote_idx], "")
                };

                // Replace literal "\n" sequences with actual newline characters.
                let processed = quoted.replace("\\n", "\n");
                // Split into lines.
                let lines: Vec<&str> = processed.lines().collect();
                let fixed_indent = "                    "; // 20 spaces for subsequent lines
                let new_content = if lines.is_empty() {
                    format!("r#\"\"#")
                } else {
                    let first_line = lines[0];
                    let mut out = format!("r#\"{}", first_line);
                    for line in lines.iter().skip(1) {
                        out.push('\n');
                        out.push_str(fixed_indent);
                        out.push_str(line);
                    }
                    out.push_str("\"#");
                    out
                };
                // Reassemble the line: original indentation + prefix + new raw string + trailing comma.
                return format!("{}{}{}{}", before, prefix, new_content, trailing);
            }
        }
    }
    // If not a content line, return unchanged.
    line.to_string()
}

/// Processes an entire RON string and, for lines within the surface array that contain
/// a `content:` field, wraps the quoted string as a raw string literal.
pub fn wrap_surface_entries(ron_str: &str) -> String {
    let mut result = String::new();
    let mut in_surface = false;

    for line in ron_str.lines() {
        let trimmed = line.trim_start();
        // Detect the start of the surface array.
        if trimmed.starts_with("surface:") {
            in_surface = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        // Detect the end of the surface array.
        if in_surface && (trimmed.starts_with("],") || trimmed.starts_with("]")) {
            in_surface = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        // When inside the surface array, look for lines containing "content:".
        if in_surface && trimmed.contains("content:") {
            let new_line = wrap_content_line(line);
            result.push_str(&new_line);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

pub fn wrap_map_entries(ron_str: &str) -> String {
    let mut result = String::new();
    let mut in_maps = false;

    for line in ron_str.lines() {
        let trimmed = line.trim_start();

        // Detect the start of the surface content.
        if trimmed.starts_with("content:") {
            in_maps = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Detect the end of the maps array.
        if in_maps && (trimmed.starts_with("],") || trimmed.starts_with("]")) {
            in_maps = false;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_maps {
            // Process only lines that look like map entry string literals.
            if trimmed.starts_with("\"") {
                let wrapped = wrap_map_entry_with_fixed_indent(line);
                result.push_str(&wrapped);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            // Outside the maps array, copy the line as-is.
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

/// Wraps a single map entry in a raw string literal.
/// Converts literal "\n" sequences into actual newlines and reindents
/// subsequent lines with a fixed indent of 18 spaces.
pub fn wrap_map_entry_with_fixed_indent(line: &str) -> String {
    // Capture the original indentation.
    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let trimmed = line.trim_start();
    if !trimmed.starts_with("\"") {
        return line.to_string();
    }
    // Remove the leading quote.
    let mut content = trimmed.strip_prefix("\"").unwrap_or(trimmed);

    // Remove the trailing quote (and optional trailing comma).
    let mut trailing = "";
    if content.ends_with("\",") {
        content = &content[..content.len() - 2];
        trailing = ",";
    } else if content.ends_with("\"") {
        content = &content[..content.len() - 1];
    }

    // Replace literal "\n" with actual newline characters.
    let content = content.replace("\\n", "\n");

    // Split the content into lines.
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return format!("{}r#\"\"#{}", indent, trailing);
    }

    // The first line remains immediately after r#"
    let first_line = lines[0];
    // For subsequent lines, use a fixed indent of 20 spaces.
    let fixed_indent = "                    "; // 20 spaces
    let mut reindented = first_line.to_string();
    for line in lines.iter().skip(1) {
        reindented.push('\n');
        reindented.push_str(fixed_indent);
        reindented.push_str(line);
    }

    // Wrap the result in a raw string literal.
    format!("{}r#\"{}\"#{}", indent, reindented, trailing)
}

pub fn insert_comment_preserve_indent_no_regex(
    ron_str: &str,
    comment: &str,
    marker: &str,
) -> String {
    let mut result = String::new();

    // Process each line individually.
    for line in ron_str.lines() {
        // Check if the trimmed line starts with the marker.
        let trimmed = line.trim_start();
        if trimmed.starts_with(marker) {
            // Capture the leading whitespace manually.
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            // Insert the comment with the same indent.
            result.push_str(&format!("{}// {}\n", indent, comment));
        }
        // Add the original line back.
        result.push_str(line);
        result.push('\n');
    }
    result
}
