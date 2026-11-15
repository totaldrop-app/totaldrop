- **Linux:** The transfer percent is written the way the system's regional
  setting (`LC_NUMERIC`) writes it, as the sizes beside it already are:
  "%50" with a Turkish region, "50 %" with a German one. It is floored with
  integer arithmetic, matching Android (#491).
