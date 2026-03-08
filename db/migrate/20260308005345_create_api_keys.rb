class CreateAPIKeys < ActiveRecord::Migration[8.1]
  def change
    create_table :api_keys do |t|
      t.references :user, null: false, foreign_key: true
      t.string :name, null: false
      t.string :client_id, null: false
      t.string :secret_key_digest, null: false
      t.datetime :expires_at
      t.datetime :last_used_at
      t.string :last_used_ip

      t.timestamps
    end
    add_index :api_keys, :client_id, unique: true
  end
end
