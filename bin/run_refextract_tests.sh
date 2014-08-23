#!/bin/sh

nosetests-2.7 refextract_regression_tests:RefextractTest \
              refextract_regression_tests:RefextractInvenioTest \
              refextract_regression_tests:TaskTest \
              refextract_unit_tests:ReTest \
              refextract_unit_tests:IbidTest \
              refextract_unit_tests:FindNumerationTest \
              refextract_unit_tests:FindSectionTest \
              refextract_unit_tests:SearchTest \
              refextract_unit_tests:RebuildReferencesTest \
              refextract_unit_tests:tagArxivTest
