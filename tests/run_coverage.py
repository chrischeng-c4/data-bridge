#!/usr/bin/env python3
"""
Run all tests with coverage reporting.

Usage:
    python -m tests.run_coverage [format] [output_file]

Formats: html, md, yaml, json, console (default: html)
"""

import sys
from data_bridge.test import run_suites_with_coverage, ReportFormat, Reporter

# Import all test suites
# Common tests (direct in tests/common/)
from tests.common.test_constraints import (
    TestConstraintClasses, TestAnnotatedTypeDetection, TestConstraintExtraction,
    TestTypeDescriptorWithConstraints, TestStringConstraintValidation,
    TestNumericConstraintValidation, TestFormatConstraintValidation, TestEdgeCases,
)
from tests.common.test_state_tracker import (
    TestStateTrackerBasics, TestStateTrackerQueries, TestStateTrackerRollback,
    TestStateTrackerReset, TestStateTrackerOriginalData, TestStateTrackerEdgeCases,
    TestStateTrackerMemoryEfficiency, TestStateTrackerPerformance,
)
from tests.common.test_connection import TestConnectionStringBuilding, TestConnectionInit

# MongoDB tests (in tests/mongo/unit/)
from tests.mongo.unit.test_security import (
    TestNoSQLInjectionPrevention, TestCollectionNameValidation,
    TestFieldNameValidation, TestErrorSanitization, TestSecurityConfiguration,
    TestSecurityIntegration,
)
from tests.mongo.unit.test_embedded import (
    TestEmbeddedDocumentBasics, TestEmbeddedSerialization, TestEmbeddedRoundTrip,
    TestEmbeddedQueries, TestEmbeddedEdgeCases,
)
from tests.mongo.unit.test_aggregation import TestAggregationHelpers, TestAggregationHelpersUnit
from tests.mongo.unit.test_inheritance import (
    TestInheritanceSetup, TestInheritanceCRUD, TestInheritanceFields, TestInheritanceEdgeCases,
)
from tests.mongo.unit.test_bulk import (
    TestBulkFastPath, TestBulkReturnType, TestBulkValidation, TestBulkCorrectness,
)
from tests.mongo.unit.test_timeseries import TestTimeSeriesConfig, TestTimeSeriesDocument, TestGranularity
from tests.mongo.unit.test_edge_cases import (
    TestLargeDocuments, TestConcurrentOperations, TestUnicodeHandling,
    TestEmptyAndNullValues, TestExtremeValues, TestConnectionRecovery,
    TestFieldNameEdgeCases,
)
from tests.mongo.unit.test_relations import (
    TestWriteRules, TestDeleteRules, TestFetchLinks, TestLinkClass, TestBackLinkClass,
)
from tests.mongo.unit.test_migrations import (
    TestMigrationBase, TestMigrationForward, TestIterativeMigration, TestFreeFallMigration,
    TestRunMigrations, TestMigrationHistory, TestMigrationErrors,
)
from tests.mongo.unit.test_queries import (
    TestQueryBuilderFilters, TestQueryBuilderSorting, TestQueryBuilderPagination, TestQueryExecution,
)
from tests.mongo.unit.test_crud import (
    TestBasicCRUD, TestUpsertOperations, TestReplaceOperations, TestDistinctOperations, TestFindAndModify,
)
from tests.mongo.unit.test_types import (
    TestBasicTypeHandling, TestDateTimeHandling, TestPydanticObjectIdBasic,
    TestTypeRoundTrip, TestBsonTypeRoundTrip, TestObjectIdRoundTrip,
)
from tests.mongo.unit.test_hooks import (
    TestBeforeInsertHook, TestAfterInsertHook, TestBeforeDeleteHook, TestMultipleHooks,
    TestAsyncHook, TestBeforeReplaceHook, TestRevisionTracking, TestStateManagement,
)
from tests.mongo.unit.test_search import (
    TestEscapeRegex, TestTextSearchUnit, TestGeoQueriesUnit,
    TestTextSearchIntegration, TestGeoQueriesIntegration, TestTransactionStubs,
)

# HTTP tests (in tests/http/unit/)
from tests.http.unit.test_client import TestHttpClientBasics, TestHttpResponse, TestHttpClientConfiguration

# All test suites
ALL_SUITES = [
    # Common tests
    TestConstraintClasses, TestAnnotatedTypeDetection, TestConstraintExtraction,
    TestTypeDescriptorWithConstraints, TestStringConstraintValidation,
    TestNumericConstraintValidation, TestFormatConstraintValidation, TestEdgeCases,
    TestStateTrackerBasics, TestStateTrackerQueries, TestStateTrackerRollback,
    TestStateTrackerReset, TestStateTrackerOriginalData, TestStateTrackerEdgeCases,
    TestStateTrackerMemoryEfficiency, TestStateTrackerPerformance,
    TestConnectionStringBuilding, TestConnectionInit,
    # MongoDB tests
    TestNoSQLInjectionPrevention, TestCollectionNameValidation,
    TestFieldNameValidation, TestErrorSanitization, TestSecurityConfiguration,
    TestSecurityIntegration,
    TestEmbeddedDocumentBasics, TestEmbeddedSerialization, TestEmbeddedRoundTrip,
    TestEmbeddedQueries, TestEmbeddedEdgeCases,
    TestAggregationHelpers, TestAggregationHelpersUnit,
    TestInheritanceSetup, TestInheritanceCRUD, TestInheritanceFields, TestInheritanceEdgeCases,
    TestBulkFastPath, TestBulkReturnType, TestBulkValidation, TestBulkCorrectness,
    TestTimeSeriesConfig, TestTimeSeriesDocument, TestGranularity,
    TestLargeDocuments, TestConcurrentOperations, TestUnicodeHandling,
    TestEmptyAndNullValues, TestExtremeValues, TestConnectionRecovery,
    TestFieldNameEdgeCases,
    TestWriteRules, TestDeleteRules, TestFetchLinks, TestLinkClass, TestBackLinkClass,
    TestMigrationBase, TestMigrationForward, TestIterativeMigration, TestFreeFallMigration,
    TestRunMigrations, TestMigrationHistory, TestMigrationErrors,
    TestQueryBuilderFilters, TestQueryBuilderSorting, TestQueryBuilderPagination, TestQueryExecution,
    TestBasicCRUD, TestUpsertOperations, TestReplaceOperations, TestDistinctOperations, TestFindAndModify,
    TestBasicTypeHandling, TestDateTimeHandling, TestPydanticObjectIdBasic,
    TestTypeRoundTrip, TestBsonTypeRoundTrip, TestObjectIdRoundTrip,
    TestBeforeInsertHook, TestAfterInsertHook, TestBeforeDeleteHook, TestMultipleHooks,
    TestAsyncHook, TestBeforeReplaceHook, TestRevisionTracking, TestStateManagement,
    TestEscapeRegex, TestTextSearchUnit, TestGeoQueriesUnit,
    TestTextSearchIntegration, TestGeoQueriesIntegration, TestTransactionStubs,
    # HTTP tests
    TestHttpClientBasics, TestHttpResponse, TestHttpClientConfiguration,
]

FORMAT_MAP = {
    'html': (ReportFormat.Html, 'test_coverage_report.html'),
    'md': (ReportFormat.Markdown, 'test_coverage_report.md'),
    'markdown': (ReportFormat.Markdown, 'test_coverage_report.md'),
    'yaml': (ReportFormat.Yaml, 'test_coverage_report.yaml'),
    'json': (ReportFormat.Json, 'test_coverage_report.json'),
    'console': (ReportFormat.Console, None),
}


def main():
    # Parse args
    format_name = sys.argv[1] if len(sys.argv) > 1 else 'html'
    output_file = sys.argv[2] if len(sys.argv) > 2 else None

    if format_name not in FORMAT_MAP:
        print(f"Unknown format: {format_name}")
        print(f"Available formats: {', '.join(FORMAT_MAP.keys())}")
        sys.exit(1)

    report_format, default_output = FORMAT_MAP[format_name]
    output_file = output_file or default_output

    print(f"Running {len(ALL_SUITES)} test suites with coverage...")
    print(f"Format: {format_name}, Output: {output_file or 'stdout'}")
    print()

    # Run with coverage
    reports = run_suites_with_coverage(
        ALL_SUITES,
        source_dirs=['python/data_bridge'],
        output_format=report_format,
        output_file=output_file,
        omit_patterns=['test_', '__pycache__'],
        verbose=True,
    )

    # For console format, print the formatted reports
    if format_name == 'console':
        reporter = Reporter(ReportFormat.Console)
        for r in reports:
            print(reporter.generate(r))


if __name__ == '__main__':
    main()
