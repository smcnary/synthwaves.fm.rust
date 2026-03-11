require "rails_helper"

RSpec.describe EPGCleanupJob do
  describe "#perform" do
    it "deletes programmes that ended more than 1 week ago" do
      old = create(:epg_programme, ends_at: 8.days.ago)

      described_class.new.perform

      expect(EPGProgramme.find_by(id: old.id)).to be_nil
    end

    it "preserves programmes that ended less than 1 week ago" do
      recent = create(:epg_programme, ends_at: 6.days.ago)

      described_class.new.perform

      expect(EPGProgramme.find_by(id: recent.id)).to be_present
    end

    it "nullifies recording references before deleting" do
      old = create(:epg_programme, ends_at: 8.days.ago)
      recording = create(:recording, epg_programme: old)

      described_class.new.perform

      expect(recording.reload.epg_programme_id).to be_nil
    end

    it "preserves current and upcoming programmes" do
      current = create(:epg_programme, starts_at: 30.minutes.ago, ends_at: 30.minutes.from_now)
      upcoming = create(:epg_programme, starts_at: 1.hour.from_now, ends_at: 2.hours.from_now)

      described_class.new.perform

      expect(EPGProgramme.find_by(id: current.id)).to be_present
      expect(EPGProgramme.find_by(id: upcoming.id)).to be_present
    end
  end
end
