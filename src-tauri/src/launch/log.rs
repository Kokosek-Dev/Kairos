const EVENT_START: &str = "<log4j:Event ";
const EVENT_END: &str = "</log4j:Event>";
const MESSAGE_OPEN: &str = "<log4j:Message><![CDATA[";
const MESSAGE_CLOSE: &str = "]]></log4j:Message>";
const THROWABLE_OPEN: &str = "<log4j:Throwable><![CDATA[";
const THROWABLE_CLOSE: &str = "]]></log4j:Throwable>";
const CDATA_SPLIT: &str = "]]]]><![CDATA[>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine {
    pub level: Option<String>,
    pub text: String,
}

/// Turns the XML events produced by Mojang's log4j client configuration back into plain lines.
/// Lines outside of an event pass through unchanged.
#[derive(Default)]
pub struct LogParser {
    event: Option<Vec<String>>,
}

impl LogParser {
    pub fn push(&mut self, line: &str) -> Vec<ParsedLine> {
        if let Some(buffer) = &mut self.event {
            buffer.push(line.to_owned());
            if line.trim() != EVENT_END {
                return Vec::new();
            }
            let event = self.event.take().unwrap_or_default().join("\n");
            return parse_event(&event);
        }
        if line.trim_start().starts_with(EVENT_START) {
            self.event = Some(vec![line.to_owned()]);
            return Vec::new();
        }
        vec![ParsedLine { level: None, text: line.to_owned() }]
    }

    pub fn flush(&mut self) -> Vec<ParsedLine> {
        self.event
            .take()
            .map(|buffer| {
                buffer
                    .into_iter()
                    .map(|text| ParsedLine { level: None, text })
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn attribute<'a>(event: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=\"");
    let start = event.find(&marker)? + marker.len();
    let end = event[start..].find('"')?;
    Some(&event[start..start + end])
}

fn between<'a>(event: &'a str, open: &str, close: &str) -> Option<String> {
    let start = event.find(open)? + open.len();
    let end = event.rfind(close)?;
    (end >= start).then(|| event[start..end].replace(CDATA_SPLIT, "]]>"))
}

fn parse_event(event: &str) -> Vec<ParsedLine> {
    let level = attribute(event, "level").unwrap_or("INFO");
    let thread = attribute(event, "thread").unwrap_or("main");
    let message = between(event, MESSAGE_OPEN, MESSAGE_CLOSE).unwrap_or_default();
    let throwable = between(event, THROWABLE_OPEN, THROWABLE_CLOSE);

    let mut lines = message.lines();
    let first = lines.next().unwrap_or_default();
    let mut parsed = vec![ParsedLine {
        level: Some(level.to_owned()),
        text: format!("[{thread}/{level}] {first}"),
    }];
    let rest = lines.map(str::to_owned).chain(
        throwable
            .iter()
            .flat_map(|text| text.lines().map(str::to_owned)),
    );
    parsed.extend(rest.map(|text| ParsedLine { level: Some(level.to_owned()), text }));
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(parser: &mut LogParser, lines: &[&str]) -> Vec<ParsedLine> {
        lines.iter().flat_map(|line| parser.push(line)).collect()
    }

    #[test]
    fn converts_an_event_into_a_readable_line_with_its_level() {
        let mut parser = LogParser::default();
        let parsed = feed(
            &mut parser,
            &[
                r#"<log4j:Event logger="fgo" timestamp="1" level="WARN" thread="Render thread">"#,
                "<log4j:Message><![CDATA[Something odd]]></log4j:Message>",
                "</log4j:Event>",
            ],
        );

        assert_eq!(
            parsed,
            vec![ParsedLine { level: Some("WARN".into()), text: "[Render thread/WARN] Something odd".into() }]
        );
    }

    #[test]
    fn keeps_multiline_messages_and_throwables() {
        let mut parser = LogParser::default();
        let parsed = feed(
            &mut parser,
            &[
                r#"<log4j:Event logger="x" timestamp="1" level="ERROR" thread="main">"#,
                "<log4j:Message><![CDATA[Crashed",
                "second line]]></log4j:Message>",
                "<log4j:Throwable><![CDATA[java.lang.Boom",
                "\tat Foo.bar]]></log4j:Throwable>",
                "</log4j:Event>",
            ],
        );

        let texts: Vec<&str> = parsed.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(texts, ["[main/ERROR] Crashed", "second line", "java.lang.Boom", "\tat Foo.bar"]);
        assert!(parsed.iter().all(|line| line.level.as_deref() == Some("ERROR")));
    }

    #[test]
    fn passes_plain_lines_through_and_flushes_unfinished_events() {
        let mut parser = LogParser::default();
        assert_eq!(parser.push("plain output")[0].text, "plain output");

        assert!(parser.push(r#"<log4j:Event logger="x" level="INFO" thread="t">"#).is_empty());
        assert_eq!(parser.flush().len(), 1);
    }
}
