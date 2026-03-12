class AddAudioChannelsToVideos < ActiveRecord::Migration[8.1]
  def change
    add_column :videos, :audio_channels, :integer
  end
end
