class AddYoutubePlaylistUrlToAlbums < ActiveRecord::Migration[8.1]
  def change
    add_column :albums, :youtube_playlist_url, :string
  end
end
