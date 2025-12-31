# THD Maker - 3D File Format Library

A comprehensive Rust library for handling various 3D manufacturing file formats including 3MF, AMF, STL, STEP (STP), and OBJ.

## Overview

THD Maker is a robust, easy-to-use library designed to read, write, and manipulate 3D manufacturing file formats. Built with Rust's safety and performance in mind.

## Features

- **Multi-format Support**: Comprehensive support for major 3D manufacturing formats
- **High Performance**: Leveraging Rust's zero-cost abstractions for optimal performance
- **Memory Safe**: Built with Rust's ownership model to prevent memory-related bugs
- **Extensible Design**: Modular architecture allowing easy addition of new formats
- **Cross-platform**: Works on Windows, macOS, and Linux

## Supported Formats

### 3MF (3D Manufacturing Format)
- [3MF Specification](https://3mf.io/spec/)
- [Official 3MF Consortium Library](https://github.com/3MFConsortium/lib3mf)
- [Library of Congress Format Description](https://www.loc.gov/preservation/digital/formats/fdd/fdd000557.shtml)

### AMF (Additive Manufacturing File Format)
ISO-52915 Standard
- [ASTM International](https://www.astm.org/)
- [ISO Standard](https://www.iso.org/standard/74640.html)
- [ISO TC261 Committee](https://committee.iso.org/home/tc261)
- [AMF XSD Schema](https://standards.iso.org/iso/52915/ISOASTM52915(2016)AMF.xsd)
- [Wikipedia: Additive Manufacturing File Format](https://en.wikipedia.org/wiki/Additive_manufacturing_file_format)

### STL (Stereolithography)
- [Wikipedia: STL Format](https://en.wikipedia.org/wiki/STL_(file_format))
- [Library of Congress Format Description](https://www.loc.gov/preservation/digital/formats/fdd/fdd000505.shtml)

### STEP/STP (Standard for the Exchange of Product Data)
ISO-10303 Standard
- [ISO Standard Overview](https://www.iso.org/standard/83105.html)
- [ISO Standard Detail](https://www.iso.org/standard/63141.html)
- [STEP Parts List](https://standards.iso.org/iso/10303/STEP_Parts_List.htm)
- [STEP Tools](https://www.steptools.com/)
- [STEP Standards Documentation](https://www.steptools.com/stds/step)
- [STEP Part 21 Edition 3](https://steptools.com/stds/step/IS_final_p21e3.html)
- [Wikipedia: ISO 10303](https://en.wikipedia.org/wiki/ISO_10303)
- [Wikipedia: ISO 10303-21](https://en.wikipedia.org/wiki/ISO_10303-21)
- [Library of Congress Format Description](https://www.loc.gov/preservation/digital/formats/fdd/fdd000448.shtml)
- [FreeCAD](https://www.freecad.org)
- [OpenCASCADE](https://www.opencascade.com)
- [OpenCASCADE Developer Portal](https://dev.opencascade.org)

### OBJ (Wavefront OBJ)
- [Library of Congress Format Description](https://www.loc.gov/preservation/digital/formats/fdd/fdd000507.shtml)
- [Wikipedia: Wavefront OBJ File](https://en.wikipedia.org/wiki/Wavefront_.obj_file)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
thdmaker = "0.0.1"
```

Or run:

```bash
cargo add thdmaker
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Thanks to all the organizations and standards bodies that maintain the 3D file format specifications
- Inspired by existing libraries in other languages
- Built with the excellent Rust ecosystem

## Links

- [Documentation](https://docs.rs/thdmaker)
- [Crates.io](https://crates.io/crates/thdmaker)
- [Changelog](CHANGELOG.md)

## References

### OBJ
- [Twinklebear/tobj](https://github.com/Twinklebear/tobj)

### STEP/STP
- [stepcode/stepcode](https://github.com/stepcode/stepcode)
- [mozman/steputils](https://github.com/mozman/steputils)
- [ricosjp/ruststep](https://github.com/ricosjp/ruststep)
- [FreeCAD/FreeCAD](https://github.com/FreeCAD/FreeCAD)