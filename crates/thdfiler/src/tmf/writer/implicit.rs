use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::implicit::*;
use super::define::package::Namespace;

impl ImplicitFunction {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("i:implicitfunction");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("identifier", self.identifier.as_str()));
        elem.push_attribute(("xmlns", Namespace::IMPLICIT_NS));

        if let Some(ref display_name) = self.display_name {
            elem.push_attribute(("displayname", display_name.as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        // Write input
        self.r#in.write(writer)?;

        // Write nodes
        for node in &self.nodes {
            node.write(writer)?;
        }

        // Write output
        self.out.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new("i:implicitfunction")))?;

        Ok(())
    }
}

impl Input {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("i:in")))?;
        for value in &self.values {
            value.write(writer)?;
        }
        writer.write_event(Event::End(BytesEnd::new("i:in")))?;
        Ok(())
    }
}

impl Output {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("i:out")))?;
        for value in &self.values {
            value.write(writer)?;
        }
        writer.write_event(Event::End(BytesEnd::new("i:out")))?;
        Ok(())
    }
}

impl Value {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = match self.r#type {
            ValueType::Ref(ref v) => v.write(),
            ValueType::Data(ref v) => v.write(),
        };
        elem.push_attribute(("identifier", self.identifier.as_str()));

        if let Some(ref display_name) = self.display_name {
            elem.push_attribute(("displayname", display_name.as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl ValueData {
    fn write(&'_ self) -> BytesStart<'_> {
        let name = match self {
            Self::Scalar => "i:scalar",
            Self::Vector => "i:vector",
            Self::Matrix => "i:matrix",
            Self::ResourceId => "i:resourceid",
        };
        let elem = BytesStart::new(name);
        elem
    }
}

impl ValueRef {
    fn write(&'_ self) -> BytesStart<'_> {
        let (name, refs) = match self {
            Self::ScalarRef(refs) => ("i:scalarref", refs),
            Self::VectorRef(refs) => ("i:vectorref", refs),
            Self::MatrixRef(refs) => ("i:matrixref", refs),
            Self::ResourceRef(refs ) => ("i:resourceidref", refs),
        };
        let mut elem = BytesStart::new(name);
        elem.push_attribute(("ref", refs.as_str()));
        elem
    }
}

impl Node {
    /// Write node to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let name = self.r#type.name();
        let mut elem = BytesStart::new(format!("i:{}", name));
        elem.push_attribute(("identifier", self.identifier.as_str()));

        if let Some(ref display_name) = self.display_name {
            elem.push_attribute(("displayname", display_name.as_str()));
        }

        if let Some(ref tag) = self.tag {
            elem.push_attribute(("tag", tag.as_str()));
        }

        let typed = self.r#type.to_string();
        let attrs = typed.split("|").collect::<Vec<_>>();
        for attr in attrs[1..].iter() {
            let kvs: Vec<_> = attr.split("=").collect();
            if kvs.len() == 2 {
                elem.push_attribute((kvs[0], kvs[1]));
            }
        }

        writer.write_event(Event::Start(elem))?;

        // Write inputs
        self.r#in.write(writer)?;

        // Write outputs
        self.out.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new(format!("i:{}", name))))?;

        Ok(())
    }
}

impl ImplicitResources {
    /// Write implicit resources to XML.
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // Write implicit functions
        for function in &self.functions {
            function.write(writer)?;
        }

        Ok(())
    }
}