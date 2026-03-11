class CreateTracksSearchIndex < ActiveRecord::Migration[8.1]
  def up
    execute <<~SQL
      CREATE VIRTUAL TABLE tracks_search USING fts5(
        track_title,
        artist_name,
        album_title,
        track_id UNINDEXED,
        tokenize='unicode61 remove_diacritics 2'
      );
    SQL

    execute <<~SQL
      INSERT INTO tracks_search (track_title, artist_name, album_title, track_id)
      SELECT tracks.title, artists.name, albums.title, tracks.id
      FROM tracks
      INNER JOIN artists ON artists.id = tracks.artist_id
      INNER JOIN albums ON albums.id = tracks.album_id;
    SQL
  end

  def down
    execute "DROP TABLE IF EXISTS tracks_search;"
  end
end
