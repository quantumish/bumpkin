use std::{io::Error, io::Write, borrow::Borrow};

use orgize::{export::{HtmlHandler, DefaultHtmlHandler, SyntectHtmlHandler}, Element};

const HTML_HEAD: &'static str = r"<head>
<link rel='stylesheet' href='./assets/latex.css'>
<script src='https://kit.fontawesome.com/76c5ce8bda.js' crossorigin='anonymous'></script>
<script type='text/x-mathjax-config'>
    MathJax.Hub.Config({
        displayAlign: 'center',
        displayIndent: '0em',
		extensions: ['[Contrib]/physics/physics.js'],
        loader: {load: ['[tex]/physics']},

        'HTML-CSS': { scale: 100,
                        linebreaks: { automatic: 'false' },
                        webFont: 'TeX'
                       },
        SVG: {scale: 100,
              linebreaks: { automatic: 'false' },
              font: 'TeX'},
        NativeMML: {scale: 100},
        TeX: { equationNumbers: {autoNumber: 'AMS'},
               MultLineWidth: '85%',
               TagSide: 'right',
               TagIndent: '.8em'
	           <!-- packages: {'[+]': ['physics']} -->
             }
});
</script>
<script src='https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.0/MathJax.js?config=TeX-AMS_HTML'></script>
</head>";

#[derive(Clone)]
struct Heading {
	name: String,
	subheadings: Vec<Heading>
}

#[derive(Default)]
struct MyHtmlHandler {
	outline: Vec<Heading>,
	inner: DefaultHtmlHandler
}

fn sublist(heading: Heading) -> maud::Markup {	
	maud::html! {
		ol {
			@for h in heading.subheadings {
				li { a href ={"#" (id_ify(h.name.clone()))} { (h.name) } }				
				@if h.subheadings.len() > 0 {
					(sublist(h))
				}
			}			
		}
	}
}

fn id_ify(name: String) -> String {
	name.to_lowercase().chars()
		.filter(|c| c.is_alphanumeric() || *c == ' ')
		.collect::<String>().replace(" ", "-")
}

impl HtmlHandler<Error> for MyHtmlHandler {
    fn start<W: Write>(&mut self, mut w: W, element: &Element) -> Result<(), Error> {
        match element {
			Element::Document { .. } => {
				write!(w, "{}", maud::html! {
					nav class="toc" {
						strong { "Outline" } 
						ol {
							@for h in self.outline.clone() {
								li {
									a href ={"#" (id_ify(h.name.clone()))} { (h.name) }
								}
								(sublist(h))
							}							
						}
						details {
							summary style="font-size: 10pt" { "Metadata" }
							code style="font-size: 7pt" { "compiled on 2/12/14" }
						}
					}
				}.into_string())?;
			},			
			Element::Title(title) => {
				write!(
					w,
					"<section id='{}'>",
					id_ify(title.raw.clone().to_string())
				)?;
                write!(w, "<h{}>", if title.level <= 6 { title.level } else { 6 })?;
            }
			Element::Section => (),
			// Element::LatexEnvironment(le) => {
			// 	let id = seahash::hash(le.contents.as_bytes());
				
			// 	std::fs::write(format!("{id}.tex"), format!(
			// 		r"\begin{{document}}
            //           {}
            //           \end{{document}}",
			// 		if le.inline {
			// 			format!(r"\({}\)", le.contents)
			// 		} else {
			// 			println!("{} {}", le.argument, le.contents);
			// 			format!(
			// 				r"\begin{{{}}}
            //                   {}
            //                 \end{{{}}}",
			// 				le.argument,
			// 				le.contents,
			// 				le.argument
			// 			)
			// 		}
			// 	));
			// 	std::process::Command::new("pdflatex")
			// 		.arg(format!("{id}.tex")).output().unwrap();
			// 	std::process::Command::new("dvisvgm")
			// 		.arg("--pdf")
			// 		.arg(format!("{id}.pdf"))
			// 		.output().unwrap();

			// 	if !le.inline { write!(w, "<br>")? }				
			// 	write!(w, "<object type='image/svg+xml' data='{}.svg' class='logo'></object>", id)?;
			// 	if !le.inline { write!(w, "<br>")? }
			// },
            _ => self.inner.start(w, element)?,
        }
        Ok(())
    }

    fn end<W: Write>(&mut self, w: W, element: &Element) -> Result<(), Error> {
        self.inner.end(w, element)
    }	
}

fn main() {
	std::env::set_var("LIBGS", "/usr/local/share/ghostscript/9.55.0/lib/libgs.dylib.9.55");
	let contents = std::fs::read_to_string("./geotherm.org").unwrap();
	let mut writer = Vec::new();
	writer.extend(HTML_HEAD.bytes());
	let mut outline = Vec::new();
	let org = orgize::Org::parse(&contents);

	writer.extend("<div id='header'>".bytes());
	for k in org.keywords() {
		println!("{}", k.key.clone());
		match k.key.clone().to_string().to_uppercase().as_str() {
			"TITLE" => writer.extend(maud::html! {
				div id="title-div" {
					title { (k.value) }
					h1 class="title" { (k.value) }
					hr id="title-hr";
				}
			}.into_string().bytes()),
			"KEYWORDS" => writer.extend(maud::html! {
				div id="keywords" hidden {
					p { i class="fas fa-tags" {} " "
						 @for k in k.value.split(" ") {
							 code { (k) }
							 " "
						 }
					}
				}						
			}.into_string().bytes()),
			"REF" => writer.extend(maud::html! {
				p {
					i class="fas fa-book" {} " "
					span style="font-size 5pt; color: gray; font-style: italic" { (k.value) }
				}
			}.into_string().bytes()),
			_ => (),
		}
	}	
	writer.extend("</div>".bytes());
	
	for (i, h) in org.headlines().enumerate() {
		let headline = h.title(&org).borrow();
		if i == 0 || headline.level == 1 {
			outline.push(Heading { name: String::from(headline.raw.clone()), subheadings: Vec::new() });
		} else {
			
			let mut current = outline.last_mut().unwrap();
			println!("{}", headline.level-1);
			for i in 0..headline.level-2 {
				println!("IN LOOP FOR {}", headline.raw);
				if let Some(c) = current.subheadings.last() {
					println!("current is {}", current.name);
					current = current.subheadings.last_mut().unwrap();
				} else { break; }
				println!("broke");
			}
			current.subheadings.push(
				Heading { name: String::from(headline.raw.clone()), subheadings: Vec::new() }
			);
		}
	}
	let handler = MyHtmlHandler { outline, inner: DefaultHtmlHandler::default() };
	let mut handler = SyntectHtmlHandler::new(handler);
    org.write_html_custom(&mut writer, &mut handler).unwrap();
    std::fs::write("test.html", writer).unwrap();
}
