class CreateRadioStations < ActiveRecord::Migration[8.1]
  def change
    create_table :radio_stations do |t|
      t.references :user, null: false, foreign_key: true
      t.string :name, null: false
      t.string :youtube_url, null: false
      t.string :youtube_video_id, null: false
      t.string :thumbnail_url
      t.text :description
      t.timestamps
    end
  end
end
