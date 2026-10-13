// meta-language:carried JavaScript export_statement (unsupported)
// | /** Browser-friendly metadata for displaying timezone conversions. */
// | export class DateTimeResult {
// |   /**
// |    * @param {{ source: string, utc: string, epochMilliseconds: number, hasDate: boolean,
// |    *   hasTime: boolean, timezone: string | null, offsetSeconds: number | null }} fields
// |    */
// |   constructor({ source, utc, epochMilliseconds, hasDate, hasTime, timezone, offsetSeconds }) {
// |     /** The calculated value in its source timezone. */
// |     this.source = source;
// |     /** The same instant formatted in UTC. */
// |     this.utc = utc;
// |     /** UTC timestamp in milliseconds since the Unix epoch. */
// |     this.epochMilliseconds = epochMilliseconds;
// |     /** Whether the original expression includes a date component. */
// |     this.hasDate = hasDate;
// |     /** Whether the original expression includes a time component. */
// |     this.hasTime = hasTime;
// |     /** Source timezone abbreviation, when known. */
// |     this.timezone = timezone;
// |     /** Source timezone offset in seconds, when known. */
// |     this.offsetSeconds = offsetSeconds;
// |   }
// |
// |   /**
// |    * Creates conversion display metadata for a timezone-aware datetime value.
// |    * @param {DateTime} dt
// |    * @returns {DateTimeResult | null}
// |    */
// |   static fromDatetime(dt) {
// |     const utc = dt.utcEquivalentDisplay();
// |     if (utc === null) {
// |       return null;
// |     }
// |     return new DateTimeResult({
// |       source: dt.toString(),
// |       utc,
// |       epochMilliseconds: dt.timestampMillis(),
// |       hasDate: dt.hasDate(),
// |       hasTime: dt.hasTime(),
// |       timezone: dt.timezoneAbbreviation(),
// |       offsetSeconds: dt.offsetSeconds(),
// |     });
// |   }
// |
// |   /** `Serialize`: serde's field names and order. */
// |   toJSON() {
// |     /** @type {Record<string, unknown>} */
// |     const json = {
// |       source: this.source,
// |       utc: this.utc,
// |       epoch_milliseconds: this.epochMilliseconds,
// |       has_date: this.hasDate,
// |       has_time: this.hasTime,
// |     };
// |     if (this.timezone !== null) {
// |       json.timezone = this.timezone;
// |     }
// |     if (this.offsetSeconds !== null) {
// |       json.offset_seconds = this.offsetSeconds;
// |     }
// |     return json;
// |   }
// | }
