-- vehicle_category_mappings now distinguishes two concepts that used to be
-- conflated in a single flat set:
--   * a ROW = a category the vehicle QUALIFIES for. The admin assigns these
--     after reviewing the vehicle info + driver documents.
--   * is_active = whether the driver has CHOSEN to currently serve that
--     category. Drivers toggle this among the categories the admin assigned.
--
-- Dispatch/matching only considers active categories. Existing rows default to
-- active so current drivers keep serving everything they already had.
ALTER TABLE vehicle_category_mappings
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;
