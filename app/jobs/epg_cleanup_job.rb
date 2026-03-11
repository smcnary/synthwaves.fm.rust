class EPGCleanupJob < ApplicationJob
  queue_as :default

  def perform
    expired_ids = EPGProgramme.where("ends_at < ?", 1.week.ago).ids
    return if expired_ids.empty?

    Recording.where(epg_programme_id: expired_ids).update_all(epg_programme_id: nil)
    EPGProgramme.where(id: expired_ids).delete_all
  end
end
