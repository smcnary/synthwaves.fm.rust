class AddDownloadStatusToTracks < ActiveRecord::Migration[8.1]
  def change
    add_column :tracks, :download_status, :string
    add_column :tracks, :download_error, :string
  end
end
