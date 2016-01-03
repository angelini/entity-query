from argparse import ArgumentParser
from faker import Factory
from random import randint

faker = Factory.create();

def gen_data(suffix, artist_count, album_count, track_count):
    artists = [(faker.name(), faker.year()) for _ in range(artist_count)]

    albums = []
    for _ in range(album_count):
        artist = artists[randint(0, artist_count - 1)]
        albums.append((faker.name(), artist[0], randint(1, 10), faker.company(), faker.year()))

    tracks = []
    for _ in range(track_count):
        album = albums[randint(0, album_count - 1)]
        artist_name = album[1]
        tracks.append((faker.last_name(), artist_name, album[0], randint(100, 300), faker.year()))

    with open('data/artists{}.csv'.format(suffix), 'w') as f:
        f.write('Name,Year\n')
        for artist in artists:
            f.write(','.join(map(str, artist)) + '\n')

    with open('data/albums{}.csv'.format(suffix), 'w') as f:
        f.write('Name,Artist,Tracks,Label,Year\n')
        for album in albums:
            f.write(','.join(map(str, album)) + '\n')

    with open('data/tracks{}.csv'.format(suffix), 'w') as f:
        f.write('Name,Artist,Album,Length,Year\n')
        for track in tracks:
            f.write(','.join(map(str, track)) + '\n')

if __name__ == '__main__':
    parser = ArgumentParser()
    parser.add_argument('--suffix', default='_fake')
    parser.add_argument('--artists', type=int, default=10)
    parser.add_argument('--albums', type=int, default=40)
    parser.add_argument('--tracks', type=int, default=100)
    args = parser.parse_args()

    gen_data(args.suffix, args.artists, args.albums, args.tracks)
