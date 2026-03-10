class AddDownloadFieldsToVideos < ActiveRecord::Migration[8.1]
  def change
    add_column :videos, :download_status, :string
    add_column :videos, :download_error, :string
    add_column :videos, :youtube_video_id, :string
    add_index :videos, :youtube_video_id
  end
end
