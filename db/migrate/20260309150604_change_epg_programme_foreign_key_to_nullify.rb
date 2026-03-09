class ChangeEPGProgrammeForeignKeyToNullify < ActiveRecord::Migration[8.1]
  def change
    remove_foreign_key :recordings, :epg_programmes
    add_foreign_key :recordings, :epg_programmes, on_delete: :nullify
  end
end
