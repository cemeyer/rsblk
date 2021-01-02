#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::Result;
use freebsd_geom::{self as geom, GeomClass};
use ptree::{self, print_config::PrintConfig, TreeBuilder};
use std::{
    self,
    default::Default,
    fmt::{self, Display, Formatter},
    io::Write,
    str::FromStr,
    string::ToString,
};
use strum_macros::Display as StrumDisplay;
use tabwriter::TabWriter;

#[derive(StrumDisplay, Debug)]
enum Col {
    #[strum(serialize = "NAME")]
    Name,
    #[strum(serialize = "CLASS")]
    Class,
    #[strum(serialize = "SIZE")]
    Size,
    #[strum(serialize = "FSTYPE")]
    FsType,
    #[strum(serialize = "LABEL")]
    Label,
    #[strum(serialize = "MOUNTPOINT")]
    Mointpoint,
}

fn emit_header(w: &mut TabWriter<Vec<u8>>, cols: &Vec<Col>) -> Result<()> {
    write!(
        w,
        "{}\n",
        cols.iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<String>>()
            .join("\t")
    )?;
    Ok(())
}

fn format_root(name: &str, ncols: usize) -> String {
    let pad = if ncols > 1 {
        "\t".repeat(ncols - 1)
    } else {
        "".to_owned()
    };
    return format!("{}{}", name, pad);
}

fn format_datum(edge: &geom::Edge, node: &geom::Geom, col: &Col) -> String {
    match col {
        Col::Name => node.name.to_owned(),
        Col::Class => format!("{:?}", &node.class),
        Col::Size => format!("{}", edge.mediasize),
        // XXX
        _ => "".to_owned(),
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct WalkState {
    inpart: bool,
    inlabel: bool,
}

// What do we actually want to find?
// (1) PART node.  We don't print the PART table itself, but we know that *direct* DEV descendents
//     are definitely things we want to print.
// (2) *Canonical* DEV nodes as well as *alias* (LABEL) DEV nodes for the same device.  We need to
//     check the mounttab for any name but will print the canonical name.  Probably.
fn walk_geom(
    forest: &geom::Graph,
    nodeid: &geom::NodeId,
    node: &geom::Geom,
    cols: &Vec<Col>,
    tb: &mut TreeBuilder,
    state: &WalkState,
) {
    for (_, edge, child) in forest.child_geoms_iter(nodeid) {
        let mut addnode = false;
        let mut newstate = *state;

        match child.class {
            GeomClass::PART => {
                newstate.inpart = true;
            }
            GeomClass::LABEL => {
                newstate.inlabel = true;
            }
            GeomClass::DEV => {
                if state.inpart && !state.inlabel {
                    addnode = true;
                }
            }
            _ => (),
        };

        if addnode {
            tb.begin_child(
                cols.iter()
                    .map(|c| format_datum(edge, child, c))
                    .collect::<Vec<String>>()
                    .join("\t"),
            );
        }

        walk_geom(forest, &edge.consumer_geom, child, cols, tb, &newstate);

        if addnode {
            tb.end_child();
        }
    }
}

// TODO:
// 1. Options parsing: [options] [device...]
fn run() -> Result<()> {
    let cols = vec![Col::Name, Col::Class, Col::Size];

    let mut tw = TabWriter::new(vec![]);
    emit_header(&mut tw, &cols)?;

    let config = PrintConfig {
        indent: 2,
        padding: 0,
        ..PrintConfig::from_env()
    };

    let forest = geom::get_graph()?;
    for (nodeid, node) in forest.roots_iter() {
        let mut tb = TreeBuilder::new(format_root(&node.name, cols.len()));
        let ws: WalkState = Default::default();
        walk_geom(&forest, nodeid, node, &cols, &mut tb, &ws);
        let tree = tb.build();
        ptree::write_tree_with(&tree, &mut tw, &config)?;
    }

    tw.flush()?;
    print!("{}", String::from_utf8(tw.into_inner()?)?);
    return Ok(());
}

fn main() {
    run().unwrap();
}
