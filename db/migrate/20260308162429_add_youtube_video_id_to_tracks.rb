class AddYoutubeVideoIdToTracks < ActiveRecord::Migration[8.1]
  def change
    add_column :tracks, :youtube_video_id, :string
    add_index :tracks, :youtube_video_id
  end
end
