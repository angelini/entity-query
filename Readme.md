# entity-query

Query data in the following form:

```
(<entity ID>, <attribute name>, <value>, <time>)
```

Sample queries:

```
e:42             # entity ID = 42
v:foo            # value contains foo
a:bar v:foo      # attribute name contains bar and the value contains foo
t:2010 | t:2011  # all datums from the year 2010 and 2011
```

CLI:

```
l <file name>                              # load file
c <file name> <entity name> <time column>  # load CSV
w <file name>                              # write file
q <query>                                  # execute query
```
