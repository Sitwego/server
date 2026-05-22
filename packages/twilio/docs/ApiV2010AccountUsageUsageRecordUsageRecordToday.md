# ApiV2010AccountUsageUsageRecordUsageRecordToday

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that accrued the usage. | [optional]
**api_version** | Option<**String**> | The API version used to create the resource. | [optional]
**as_of** | Option<**String**> | Usage records up to date as of this timestamp, formatted as YYYY-MM-DDTHH:MM:SS+00:00. All timestamps are in GMT | [optional]
**category** | Option<**String**> | The category of usage. For more information, see [Usage Categories](https://www.twilio.com/docs/usage/api/usage-record#usage-categories). | [optional]
**count** | Option<**String**> | The number of usage events, such as the number of calls. | [optional]
**count_unit** | Option<**String**> | The units in which `count` is measured, such as `calls` for calls or `messages` for SMS. | [optional]
**description** | Option<**String**> | A plain-language description of the usage category. | [optional]
**end_date** | Option<[**String**](String.md)> | The last date for which usage is included in the UsageRecord. The date is specified in GMT and formatted as `YYYY-MM-DD`. | [optional]
**price** | Option<**f64**> | The total price of the usage in the currency specified in `price_unit` and associated with the account. | [optional]
**price_unit** | Option<**String**> | The currency in which `price` is measured, in [ISO 4127](https://www.iso.org/iso/home/standards/currency_codes.htm) format, such as `usd`, `eur`, and `jpy`. | [optional]
**start_date** | Option<[**String**](String.md)> | The first date for which usage is included in this UsageRecord. The date is specified in GMT and formatted as `YYYY-MM-DD`. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their URIs. For more information, see [List Subresources](https://www.twilio.com/docs/usage/api/usage-record#list-subresources). | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**usage** | Option<**String**> | The amount used to bill usage and measured in units described in `usage_unit`. | [optional]
**usage_unit** | Option<**String**> | The units in which `usage` is measured, such as `minutes` for calls or `messages` for SMS. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


