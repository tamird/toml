extern crate toml_edit;

macro_rules! parse_key {
    ($s:expr) => (
        {
            let key = $s.parse::<Key>();
            assert!(key.is_ok());
            &key.unwrap()
        }
    );
}

macro_rules! as_table {
    ($e:ident) => (
        {
            assert!($e.is_table());
            $e.as_table_mut().unwrap()
        }
    );
}

// rusfmt, U Can't Touch This
#[cfg(test)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod tests {
    use toml_edit::{Document, Key, Value, Table, value, table, array};
    use std::iter::FromIterator;

    struct Test {
        doc: Document,
    }

    fn given(input: &str) -> Test {
        let doc = input.parse::<Document>();
        assert!(doc.is_ok());
        Test {
            doc: doc.unwrap(),
        }
    }

    impl Test {
        fn running<F>(&mut self, func: F) -> &mut Self
            where F: Fn(&mut Table)
        {
            {
                let root = self.doc.as_table_mut();
                func(root);
            }
            self
        }

        fn produces(&self, expected: &str) {
            assert_eq!(self.doc.to_string(), expected);
        }
    }

// insertion

#[test]
fn test_insert_leaf_table() {
    given(r#"
        [servers]

        [servers.alpha]
        ip = "10.0.0.1"
        dc = "eqdc10"

        [other.table]"#
    ).running(|root| {
        root["servers"]["beta"] = table();
        root["servers"]["beta"]["ip"] = value("10.0.0.2");
        root["servers"]["beta"]["dc"] = value("eqdc10");
    }).produces(r#"
        [servers]

        [servers.alpha]
        ip = "10.0.0.1"
        dc = "eqdc10"

[servers.beta]
ip = "10.0.0.2"
dc = "eqdc10"

        [other.table]
"#
    );
}

#[test]
fn test_insert_nonleaf_table() {
    given(r#"
        [other.table]"#
    ).running(|root| {
        root["servers"] = table();
        root["servers"]["alpha"] = table();
        root["servers"]["alpha"]["ip"] = value("10.0.0.1");
        root["servers"]["alpha"]["dc"] = value("eqdc10");
    }).produces(r#"
        [other.table]

[servers]

[servers.alpha]
ip = "10.0.0.1"
dc = "eqdc10"
"#
    );
}

#[test]
fn test_insert_array() {
    given(r#"
        [package]
        title = "withoutarray""#
    ).running(|root| {
        root["bin"] = array();
        assert!(root["bin"].is_array_of_tables());
        let array = root["bin"].as_array_of_tables_mut().unwrap();
        {
            let first = array.append(Table::new());
            first["hello"] = value("world");
        }
        array.append(Table::new());
    }).produces(r#"
        [package]
        title = "withoutarray"

[[bin]]
hello = "world"

[[bin]]
"#
    );
}


#[test]
fn test_insert_values() {
    given(r#"
        [tbl.son]"#
    ).running(|root| {
        root["tbl"]["key1"] = value("value1");
        root["tbl"]["\"key2\""] = value(42);
        root["tbl"]["'key3'"] = value(8.1415926);
    }).produces(r#"
[tbl]
key1 = "value1"
"key2" = 42
'key3' = 8.1415926

        [tbl.son]
"#
    );
}

// removal

#[test]
fn test_remove_leaf_table() {
    given(r#"
        [servers]

        # Indentation (tabs and/or spaces) is allowed but not required
[servers.alpha]
        ip = "10.0.0.1"
        dc = "eqdc10"

        [servers.beta]
        ip = "10.0.0.2"
        dc = "eqdc10""#
    ).running(|root| {
        let servers = root.entry("servers");
        let servers = as_table!(servers);
        assert!(servers.remove("alpha").is_some());
    }).produces(r#"
        [servers]

        [servers.beta]
        ip = "10.0.0.2"
        dc = "eqdc10"
"#
    );
}


#[test]
fn test_remove_nonleaf_table() {
    given(r#"
        title = "not relevant"

        # comment 1
        [a.b.c] # comment 1.1
        key1 = 1 # comment 1.2
        # comment 2
        [b] # comment 2.1
        key2 = 2 # comment 2.2

        # comment 3
        [a] # comment 3.1
        key3 = 3 # comment 3.2
        [[a.'array']]
        b = 1

        [[a.b.c.trololololololo]] # ohohohohoho
        c = 2
        key3 = 42

           # comment on some other table
           [some.other.table]




        # comment 4
        [a.b] # comment 4.1
        key4 = 4 # comment 4.2
        key41 = 41 # comment 4.3


    "#).running(|root| {
        assert!(root.remove("a").is_some());
    }).produces(r#"
        title = "not relevant"
        # comment 2
        [b] # comment 2.1
        key2 = 2 # comment 2.2

           # comment on some other table
           [some.other.table]


    "#);
}

#[test]
fn test_remove_array_entry() {
    given(r#"
        [package]
        name = "hello"
        version = "1.0.0"

        [[bin]]
        name = "world"
        path = "src/bin/world/main.rs"

        [dependencies]
        nom = "4.0" # future is here

        [[bin]]
        name = "delete me please"
        path = "src/bin/dmp/main.rs""#
    ).running(|root| {
        let dmp = root.entry("bin");
        assert!(dmp.is_array_of_tables());
        let dmp = dmp.as_array_of_tables_mut().unwrap();
        assert_eq!(dmp.len(), 2);
        dmp.remove(1);
        assert_eq!(dmp.len(), 1);
    }).produces(r#"
        [package]
        name = "hello"
        version = "1.0.0"

        [[bin]]
        name = "world"
        path = "src/bin/world/main.rs"

        [dependencies]
        nom = "4.0" # future is here
"#
    );
}

#[test]
fn test_remove_array() {
    given(r#"
        [package]
        name = "hello"
        version = "1.0.0"

        [[bin]]
        name = "world"
        path = "src/bin/world/main.rs"

        [dependencies]
        nom = "4.0" # future is here

        [[bin]]
        name = "delete me please"
        path = "src/bin/dmp/main.rs""#
    ).running(|root| {
        assert!(root.remove("bin").is_some());
    }).produces(r#"
        [package]
        name = "hello"
        version = "1.0.0"

        [dependencies]
        nom = "4.0" # future is here
"#
    );
}


#[test]
fn test_remove_value() {
    given(r#"
        name = "hello"
        # delete this
        version = "1.0.0" # please
        documentation = "https://docs.rs/hello""#
    ).running(|root| {
        let value = root.remove("version");
        assert!(value.is_some());
        let value = value.unwrap();
        assert!(value.is_value());
        let value = value.as_value().unwrap();
        assert!(value.is_str());
        let value = value.as_str().unwrap();
        assert_eq!(value, "1.0.0");
    }).produces(r#"
        name = "hello"
        documentation = "https://docs.rs/hello"
"#
    );
}

#[test]
fn test_remove_last_value_from_implicit() {
    given(r#"
        [a]
        b = 1"#
    ).running(|root| {
        let a = root.entry("a");
        assert!(a.is_table());
        let a = as_table!(a);
        a.set_implicit(true);
        let value = a.remove("b");
        assert!(value.is_some());
        let value = value.unwrap();
        assert!(value.is_value());
        let value = value.as_value().unwrap();
        assert_eq!(value.as_integer(), Some(1));
    }).produces(r#""#);
}

// values

#[test]
fn test_sort_values() {
    given(r#"
        [a.z]

        [a]
        # this comment is attached to b
        b = 2 # as well as this
        a = 1
        c = 3

        [a.y]"#
    ).running(|root| {
        let a = root.entry("a");
        let a = as_table!(a);
        a.sort_values();
    }).produces(r#"
        [a]
        a = 1
        # this comment is attached to b
        b = 2 # as well as this
        c = 3

        [a.z]

        [a.y]
"#
    );
}

macro_rules! as_array {
    ($entry:ident) => (
        {
            assert!($entry.is_value());
            let a = $entry.as_value_mut().unwrap();
            assert!(a.is_array());
            a.as_array_mut().unwrap()
        }
    );
}

#[test]
fn test_insert_into_array() {
    given(r#"
        a = [1,2,3]
        b = []"#
    ).running(|root| {
        {
            let a = root.entry("a");
            let a = as_array!(a);
            assert_eq!(a.len(), 3);
            assert!(a.get(2).is_some());
            assert!(a.push(4));
            assert_eq!(a.len(), 4);
            a.fmt();
        }
        let b = root.entry("b");
        let b = as_array!(b);
        assert!(b.is_empty());
        assert!(b.push("hello"));
        assert_eq!(b.len(), 1);
    }).produces(r#"
        a = [1, 2, 3, 4]
        b = ["hello"]
"#
    );
}

#[test]
fn test_remove_from_array() {
    given(r#"
        a = [1, 2, 3, 4]
        b = ["hello"]"#
    ).running(|root| {
        {
            let a = root.entry("a");
            let a = as_array!(a);
            assert_eq!(a.len(), 4);
            assert!(a.remove(3).is_integer());
            assert_eq!(a.len(), 3);
        }
        let b = root.entry("b");
        let b = as_array!(b);
        assert_eq!(b.len(), 1);
        assert!(b.remove(0).is_str());
        assert!(b.is_empty());
    }).produces(r#"
        a = [1, 2, 3]
        b = []
"#
    );
}

macro_rules! as_inline_table {
    ($entry:ident) => (
        {
            assert!($entry.is_value());
            let a = $entry.as_value_mut().unwrap();
            assert!(a.is_inline_table());
            a.as_inline_table_mut().unwrap()
        }
    );
}

#[test]
fn test_insert_into_inline_table() {
    given(r#"
        a = {a=2,  c = 3}
        b = {}"#
    ).running(|root| {
        {
            let a = root.entry("a");
            let a = as_inline_table!(a);
            assert_eq!(a.len(), 2);
            assert!(a.contains_key("a") && a.get("c").is_some());
            a.get_or_insert("b", 42);
            assert_eq!(a.len(), 3);
            a.fmt();
        }
        let b = root.entry("b");
        let b = as_inline_table!(b);
        assert!(b.is_empty());
        b.get_or_insert("'hello'", "world");
        assert_eq!(b.len(), 1);
        b.fmt()
    }).produces(r#"
        a = { a = 2, c = 3, b = 42 }
        b = { 'hello' = "world" }
"#
    );
}

#[test]
fn test_remove_from_inline_table() {
    given(r#"
        a = {a=2,  c = 3, b = 42}
        b = {'hello' = "world"}"#
    ).running(|root| {
        {
            let a = root.entry("a");
            let a = as_inline_table!(a);
            assert_eq!(a.len(), 3);
            assert!(a.remove("c").is_some());
            assert_eq!(a.len(), 2);
        }
        let b = root.entry("b");
        let b = as_inline_table!(b);
        assert_eq!(b.len(), 1);
        assert!(b.remove("hello").is_some());
        assert!(b.is_empty());
    }).produces(r#"
        a = {a=2, b = 42}
        b = {}
"#
    );
}

#[test]
fn test_inline_table_append() {
    let mut a = Value::from_iter(vec![
        (parse_key!("a"), 1),
        (parse_key!("b"), 2),
        (parse_key!("c"), 3),
    ]);
    let a = a.as_inline_table_mut().unwrap();

    let mut b = Value::from_iter(vec![
        (parse_key!("c"), 4),
        (parse_key!("d"), 5),
        (parse_key!("e"), 6),
    ]);
    let b = b.as_inline_table_mut().unwrap();

    b.merge_into(a);
    assert_eq!(a.len(), 5);
    assert!(a.contains_key("e"));
    assert!(b.is_empty());
}

} // mod tests
