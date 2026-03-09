class CreateVideos < ActiveRecord::Migration[8.1]
  def change
    create_table :videos do |t|
      t.references :user, null: false, foreign_key: true
      t.string :title, null: false
      t.text :description
      t.float :duration
      t.integer :width
      t.integer :height
      t.string :file_format
      t.bigint :file_size
      t.string :video_codec
      t.string :audio_codec
      t.integer :bitrate
      t.string :status, default: "processing", null: false
      t.string :error_message

      t.timestamps
    end
  end
end
