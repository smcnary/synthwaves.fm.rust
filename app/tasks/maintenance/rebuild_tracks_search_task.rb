module Maintenance
  class RebuildTracksSearchTask < MaintenanceTasks::Task
    def collection
      Track.includes(:artist, :album)
    end

    def count
      Track.count
    end

    def process(track)
      ActiveRecord::Base.connection.execute(
        ActiveRecord::Base.sanitize_sql_array([
          "DELETE FROM tracks_search WHERE track_id = ?", track.id.to_s
        ])
      )
      ActiveRecord::Base.connection.execute(
        ActiveRecord::Base.sanitize_sql_array([
          "INSERT INTO tracks_search (track_title, artist_name, album_title, track_id) VALUES (?, ?, ?, ?)",
          track.title, track.artist.name, track.album.title, track.id
        ])
      )
    end
  end
end
