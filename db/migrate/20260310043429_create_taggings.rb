class CreateTaggings < ActiveRecord::Migration[8.1]
  def change
    create_table :taggings do |t|
      t.references :tag, null: false, foreign_key: true
      t.string :taggable_type, null: false
      t.bigint :taggable_id, null: false
      t.references :user, null: false, foreign_key: true

      t.timestamps
    end

    add_index :taggings, [:tag_id, :taggable_type, :taggable_id, :user_id], unique: true, name: "index_taggings_uniqueness"
    add_index :taggings, [:taggable_type, :taggable_id]
  end
end
