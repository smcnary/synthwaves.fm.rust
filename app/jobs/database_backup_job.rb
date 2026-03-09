class DatabaseBackupJob < ApplicationJob
  queue_as :default

  def perform
    DatabaseBackupService.call
  end
end
