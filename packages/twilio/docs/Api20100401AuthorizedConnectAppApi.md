# \Api20100401AuthorizedConnectAppApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_authorized_connect_app**](Api20100401AuthorizedConnectAppApi.md#fetch_authorized_connect_app) | **GET** /2010-04-01/Accounts/{AccountSid}/AuthorizedConnectApps/{ConnectAppSid}.json | Fetch an instance of an authorized-connect-app
[**list_authorized_connect_app**](Api20100401AuthorizedConnectAppApi.md#list_authorized_connect_app) | **GET** /2010-04-01/Accounts/{AccountSid}/AuthorizedConnectApps.json | Retrieve a list of authorized-connect-apps belonging to the account used to make the request



## fetch_authorized_connect_app

> models::ApiV2010AccountAuthorizedConnectApp fetch_authorized_connect_app(account_sid, connect_app_sid)
Fetch an instance of an authorized-connect-app

Fetch an instance of an authorized-connect-app

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the AuthorizedConnectApp resource to fetch. | [required] |
**connect_app_sid** | **String** | The SID of the Connect App to fetch. | [required] |

### Return type

[**models::ApiV2010AccountAuthorizedConnectApp**](api.v2010.account.authorized_connect_app.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_authorized_connect_app

> models::ListAuthorizedConnectAppResponse list_authorized_connect_app(account_sid, page_size, page, page_token)
Retrieve a list of authorized-connect-apps belonging to the account used to make the request

Retrieve a list of authorized-connect-apps belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the AuthorizedConnectApp resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListAuthorizedConnectAppResponse**](ListAuthorizedConnectAppResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

