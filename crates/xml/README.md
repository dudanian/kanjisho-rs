# Yet another XML library

This crate provides a mostly XML 1.0 compliant parser. It is heavily inspired by
the Golang XML parser. Which is to say I wanted to use it like I would the
following Go code where `DecodeElement` is basically just a Serde deserialize:

```go
d := xml.NewDecoder(r)
for {
	token, err := d.Token()
	if err != nil {
		return err
	}

	switch elem := token.(type) {
	case xml.StartElement:
		if elem.Name.Local == "entry" {
			return d.DecodeElement(entry, &elem)
		}
	}
}
```

## But why?

So why not just use one of either `xml-rs` or `quick-xml`? To put it simply: DTD
parsing support. `xml-rs` allows you to add custom entities, but does not
support automatically parsing them from the DTD. `quick-xml` just doesn't
support DTD parsing or custom entities at all.

So then what are the requirements of this XML library?
* two usage patters like with Golang:
  * token iteration support
  * serde deserialization support (specifically after token iteration)
* DTD custom entitiy support
  * parsing a DTD without errors (including internal subset)
  * reading custom entities from DTD
  * replacing custom entities in XML text

Another more petty reason for this library existing is because I thought writing
it would be fun :). I sure learned a lot about Rust while writing this parser.

## Compatibility

This is only MOSTLY XML-1.0 compliant. Some limitations include:
* only UTF-8 support (no UTF-16)
* does not verify UTF-8 text is confined to supported char sets
* does not verify parsed elements against DTD schema

Some other noteworthy limitations:
* does not pass the following data to the user:
  * comment data, completely ignores comments
  * DTD declarations, parses it all internally
  * whitespace outside of root element
* DTD parsing
  * does not check external entities

Some of these limitations might change in the future.