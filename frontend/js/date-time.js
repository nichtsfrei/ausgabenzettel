// Source - https://stackoverflow.com/a
// Posted by Cheery, modified by community. See post 'Timeline' for change history
// Retrieved 2025-12-31, License - CC BY-SA 4.0

/**
 * Returns the week number for this date.
 * "starts" on for your locale - it can be from 0 to 6. If dowOffset is 1 (Monday),
 * the week returned is the ISO 8601 week number.
 * @param int dowOffset
 * @return int
 */
Date.prototype.getWeek = function () {
  /*getWeek() was developed by Nick Baicoianu at MeanFreePath: http://www.meanfreepath.com */

  let get_day_of_year = function (year) {
    let newYear = new Date(year, 0, 1);
    let day = newYear.getDay() - 1; // we start on monday
    return day >= 0 ? day : day + 7;
  };
  let that = this;
  let get_day_num_of_year = function (year) {
    let newYear = new Date(year, 0, 1);
    return (
      Math.floor(
        (that.getTime() -
          newYear.getTime() -
          (that.getTimezoneOffset() - newYear.getTimezoneOffset()) * 60000) /
          86400000,
      ) + 1
    );
  };
  let day = get_day_of_year(this.getFullYear()); //the day of week the year begins on
  let daynum = get_day_num_of_year(this.getFullYear());
  let weeknum = Math.floor((daynum + day - 1) / 7);

  if (day < 4) {
    if (weeknum + 1 > 52) {
      /*if the next year starts before the middle of
              the week, it is week #1 of that year*/
      return get_day_of_year(this.getFullYear() + 1) < 4 ? 1 : 53;
    }
  }

  //if the year starts before the middle of a week
  return weeknum;
};
