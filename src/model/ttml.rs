#[cfg(test)]
mod tests {
    use lyrics_helper_core::{ContentType, TtmlParsingOptions};
    use ttml_processor::parse_ttml;

    #[test]
    fn test_ttml_parsing_accuracy() {
        let ttml_content = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <tt xmlns="http://www.w3.org/ns/ttml"
                xmlns:tts="http://www.w3.org/ns/ttml#styling"
                xmlns:itunes="http://itunes.apple.com/lyric-ttml-extensions"
                xmlns:ttm="http://www.w3.org/ns/ttml#metadata"
                xml:lang="en-US">
              <head>
                <metadata>
                  <ttm:title>City of Stars</ttm:title>
                  <ttm:agent type="person" xml:id="v1">
                      <ttm:name type="full">Ryan Gosling</ttm:name>
                  </ttm:agent>
                  <ttm:agent type="person" xml:id="v2">
                      <ttm:name type="full">Emma Stone</ttm:name>
                  </ttm:agent>
                  <ttm:agent type="group" xml:id="v3"/>
                </metadata>
              </head>
              <body dur="02:29.720">
                <div begin="00:09.327" end="00:44.651" itunes:song-part="Verse">
                  <p begin="00:09.327" end="00:12.109" ttm:agent="v1">City of stars</p>
                  <p begin="00:12.426" end="00:15.906" ttm:agent="v1">Are you shining just for me?</p>
                  <p begin="00:18.972" end="00:21.753" ttm:agent="v1">City of stars</p>
                  <p begin="00:22.024" end="00:25.552" ttm:agent="v1">There's so much that I can't see</p>
                  <p begin="00:28.052" end="00:30.752" ttm:agent="v1">Who knows?</p>
                  <p begin="00:31.373" end="00:37.357" ttm:agent="v1">I felt it from the first embrace I shared with you</p>
                  <p begin="00:37.803" end="00:41.859" ttm:agent="v2">That now our dreams</p>
                  <p begin="00:41.957" end="00:44.651" ttm:agent="v2">May finally come true</p>
                </div>
                <div begin="00:48.026" end="01:14.408" itunes:song-part="Verse">
                  <p begin="00:48.026" end="00:50.585" ttm:agent="v2">City of stars</p>
                  <p begin="00:50.757" end="00:54.257" ttm:agent="v2">Just one thing everybody wants</p>
                  <p begin="00:57.199" end="00:59.753" ttm:agent="v2">There in the bars</p>
                  <p begin="00:59.753" end="01:05.401" ttm:agent="v2">And through the smokescreen of the crowded restaurants</p>
                  <p begin="01:05.637" end="01:11.699" ttm:agent="v2">It's love, yes, all we're looking for is love</p>
                  <p begin="01:11.699" end="01:14.408" ttm:agent="v2">From someone else</p>
                </div>
                <div begin="01:15.080" end="01:54.303" itunes:song-part="Chorus">
                  <p begin="01:15.080" end="01:16.660" ttm:agent="v1">A rush</p>
                  <p begin="01:16.159" end="01:17.555" ttm:agent="v2">A glance</p>
                  <p begin="01:17.304" end="01:18.858" ttm:agent="v1">A touch</p>
                  <p begin="01:18.357" end="01:19.705" ttm:agent="v2">A dance</p>
                  <p begin="01:19.705" end="01:22.873" ttm:agent="v3">Look in somebody's eyes</p>
                  <p begin="01:22.874" end="01:25.029" ttm:agent="v3">To light up the skies</p>
                  <p begin="01:25.030" end="01:28.223" ttm:agent="v3">To open the world and send it reeling</p>
                  <p begin="01:28.324" end="01:34.000" ttm:agent="v3">A voice that says, "I'll be here, and you'll be alright"</p>
                  <p begin="01:37.001" end="01:39.905" ttm:agent="v3">I don't care if I know</p>
                  <p begin="01:40.003" end="01:42.126" ttm:agent="v3">Just where I will go</p>
                  <p begin="01:42.126" end="01:45.302" ttm:agent="v3">'Cause all that I need's this crazy feeling</p>
                  <p begin="01:45.304" end="01:48.958" ttm:agent="v3">A rat, tat, tat on my heart</p>
                  <p begin="01:49.703" end="01:54.303" ttm:agent="v1">Think I want it to stay</p>
                </div>
                <div begin="01:55.855" end="02:16.755" itunes:song-part="Outro">
                  <p begin="01:55.855" end="01:58.807" ttm:agent="v1">City of stars</p>
                  <p begin="01:59.104" end="02:02.506" ttm:agent="v1">Are you shining just for me?</p>
                  <p begin="01:05.936" end="02:08.957" ttm:agent="v1">City of stars</p>
                  <p begin="02:10.380" end="02:16.755" ttm:agent="v2">You never shined so brightly</p>
                </div>
              </body>
            </tt>
            "#;

        let options = TtmlParsingOptions::default();
        let parsed_data = parse_ttml(ttml_content, &options).expect("Failed to parse TTML");

        // 1. Verify Line Count
        // There are 31 <p> tags in the provided TTML string.
        assert_eq!(
            parsed_data.lines.len(),
            31,
            "Total parsed lines should equal 31"
        );

        // 2. Verify First Line
        let first_line = &parsed_data.lines[0];
        // 00:09.327 converts to 9327 ms
        assert_eq!(first_line.start_ms, 9327);

        let main_track = first_line
            .tracks
            .iter()
            .find(|t| t.content_type == ContentType::Main)
            .expect("Main track missing on first line");

        // Line is "City of stars" -> 3 words
        assert_eq!(main_track.content.words.len(), 3);

        let first_word = &main_track.content.words[0];
        // Depending on your tokenizer, the first word is "City"
        assert_eq!(first_word.syllables[0].text, "City");
        assert_eq!(first_word.syllables[0].start_ms, 9327);

        // 3. Verify Last Line (sanity check for full parsing)
        let last_line = parsed_data.lines.last().unwrap();
        // 02:10.380 converts to 130380 ms
        assert_eq!(last_line.start_ms, 130380);

        println!("Successfully parsed TTML!");
    }
}
