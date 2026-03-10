class CreateTags < ActiveRecord::Migration[8.1]
  def change
    create_table :tags do |t|
      t.string :name, null: false
      t.string :tag_type, null: false

      t.timestamps
    end

    add_index :tags, [:name, :tag_type], unique: true
  end
end
