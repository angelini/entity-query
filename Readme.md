# entity-query

Query data in the following form:

```
(<entity ID>, <attribute name>, <value>, <time>)
```

Sample queries:

```
e=42             # entity ID = 42
v:foo            # value contains foo
a:bar v:foo      # attribute name contains bar and the value contains foo
t>1970           # occured after 1970
t>=1970          # occured after or in 1970
t=2010 | t=2011  # all datums from the year 2010 and 2011
```

CLI:

```
l <file name>                                        # load file
c <file name> <entity name> <time column> [<join>]*  # load CSV
w <file name>                                        # write file
q <query>                                            # execute query
```

Sample sessions:

```
$ cargo run

> c data/artists.csv artist Year
# Load the artists CSV into a fresh DB

> c data/albums.csv album Year join(Artist, "a=artist/name")
# Load the albums CSV and join the Artists column on the results of "a=artist/name"

> c data/tracks.csv track Year join(Artist, "a=artist/name") join(Album, "a=album/name")
# Load the tracks CSV and join both the Artists and Album columns

> q e=12
(12, track/name, Flaming, 1967)
(12, track/artist, Pink Floyd, 1967)
(12, track/album, The Piper at the Gates of Dawn, 1967)
(12, track/length, 166, 1967)

> q e:(a=artist/name v:Led) a=album/name
# Query for all the album names where the artist name contains Led
(5, album/name, Led Zeppelin II, 1969)
(6, album/name, Houses of the Holy, 1973)
```
