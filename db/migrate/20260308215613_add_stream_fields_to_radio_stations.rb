class AddStreamFieldsToRadioStations < ActiveRecord::Migration[8.1]
  def change
    add_column :radio_stations, :source_type, :string, null: false, default: "youtube"
    add_column :radio_stations, :stream_url, :string
    add_column :radio_stations, :original_url, :string

    change_column_null :radio_stations, :youtube_url, true
    change_column_null :radio_stations, :youtube_video_id, true
  end
end
