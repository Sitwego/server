# \Api20100401MediaApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_media**](Api20100401MediaApi.md#list_media) | **GET** /2010-04-01/Accounts/{AccountSid}/Messages/{MessageSid}/Media.json | Read a list of Media resources associated with a specific Message resource



## list_media

> models::ListMediaResponse list_media(account_sid, message_sid, date_created, date_created_less_than, date_created_greater_than, page_size, page, page_token)
Read a list of Media resources associated with a specific Message resource

Read a list of Media resources associated with a specific Message resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is associated with the Media resources. | [required] |
**message_sid** | **String** | The SID of the Message resource that is associated with the Media resources. | [required] |
**date_created** | Option<**String**> | Only include Media resources that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read Media that were created on this date. You can also specify an inequality, such as `StartTime<=YYYY-MM-DD`, to read Media that were created on or before midnight of this date, and `StartTime>=YYYY-MM-DD` to read Media that were created on or after midnight of this date. |  |
**date_created_less_than** | Option<**String**> | Only include Media resources that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read Media that were created on this date. You can also specify an inequality, such as `StartTime<=YYYY-MM-DD`, to read Media that were created on or before midnight of this date, and `StartTime>=YYYY-MM-DD` to read Media that were created on or after midnight of this date. |  |
**date_created_greater_than** | Option<**String**> | Only include Media resources that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read Media that were created on this date. You can also specify an inequality, such as `StartTime<=YYYY-MM-DD`, to read Media that were created on or before midnight of this date, and `StartTime>=YYYY-MM-DD` to read Media that were created on or after midnight of this date. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListMediaResponse**](ListMediaResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

