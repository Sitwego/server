# \Api20100401ShortCodeApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_short_code**](Api20100401ShortCodeApi.md#fetch_short_code) | **GET** /2010-04-01/Accounts/{AccountSid}/SMS/ShortCodes/{Sid}.json | Fetch an instance of a short code
[**list_short_code**](Api20100401ShortCodeApi.md#list_short_code) | **GET** /2010-04-01/Accounts/{AccountSid}/SMS/ShortCodes.json | Retrieve a list of short-codes belonging to the account used to make the request
[**update_short_code**](Api20100401ShortCodeApi.md#update_short_code) | **POST** /2010-04-01/Accounts/{AccountSid}/SMS/ShortCodes/{Sid}.json | Update a short code with the following parameters



## fetch_short_code

> models::ApiV2010AccountShortCode fetch_short_code(account_sid, sid)
Fetch an instance of a short code

Fetch an instance of a short code

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ShortCode resource(s) to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the ShortCode resource to fetch | [required] |

### Return type

[**models::ApiV2010AccountShortCode**](api.v2010.account.short_code.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_short_code

> models::ListShortCodeResponse list_short_code(account_sid, friendly_name, short_code, page_size, page, page_token)
Retrieve a list of short-codes belonging to the account used to make the request

Retrieve a list of short-codes belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ShortCode resource(s) to read. | [required] |
**friendly_name** | Option<**String**> | The string that identifies the ShortCode resources to read. |  |
**short_code** | Option<**String**> | Only show the ShortCode resources that match this pattern. You can specify partial numbers and use '*' as a wildcard for any digit. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListShortCodeResponse**](ListShortCodeResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_short_code

> models::ApiV2010AccountShortCode update_short_code(account_sid, sid, friendly_name, api_version, sms_url, sms_method, sms_fallback_url, sms_fallback_method)
Update a short code with the following parameters

Update a short code with the following parameters

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ShortCode resource(s) to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the ShortCode resource to update | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe this resource. It can be up to 64 characters long. By default, the `FriendlyName` is the short code. |  |
**api_version** | Option<**String**> | The API version to use to start a new TwiML session. Can be: `2010-04-01` or `2008-08-01`. |  |
**sms_url** | Option<**String**> | The URL we should call when receiving an incoming SMS message to this short code. |  |
**sms_method** | Option<**String**> | The HTTP method we should use when calling the `sms_url`. Can be: `GET` or `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call if an error occurs while retrieving or executing the TwiML from `sms_url`. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method that we should use to call the `sms_fallback_url`. Can be: `GET` or `POST`. |  |

### Return type

[**models::ApiV2010AccountShortCode**](api.v2010.account.short_code.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

