# \Api20100401CredentialListApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_credential_list**](Api20100401CredentialListApi.md#create_sip_credential_list) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists.json | Create a Credential List
[**delete_sip_credential_list**](Api20100401CredentialListApi.md#delete_sip_credential_list) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{Sid}.json | Delete a Credential List
[**fetch_sip_credential_list**](Api20100401CredentialListApi.md#fetch_sip_credential_list) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{Sid}.json | Get a Credential List
[**list_sip_credential_list**](Api20100401CredentialListApi.md#list_sip_credential_list) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists.json | Get All Credential Lists
[**update_sip_credential_list**](Api20100401CredentialListApi.md#update_sip_credential_list) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/CredentialLists/{Sid}.json | Update a Credential List



## create_sip_credential_list

> models::ApiV2010AccountSipSipCredentialList create_sip_credential_list(account_sid, friendly_name)
Create a Credential List

Create a Credential List

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**friendly_name** | **String** | A human readable descriptive text that describes the CredentialList, up to 64 characters long. | [required] |

### Return type

[**models::ApiV2010AccountSipSipCredentialList**](api.v2010.account.sip.sip_credential_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_credential_list

> delete_sip_credential_list(account_sid, sid)
Delete a Credential List

Delete a Credential List

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**sid** | **String** | The credential list Sid that uniquely identifies this resource | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_credential_list

> models::ApiV2010AccountSipSipCredentialList fetch_sip_credential_list(account_sid, sid)
Get a Credential List

Get a Credential List

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**sid** | **String** | The credential list Sid that uniquely identifies this resource | [required] |

### Return type

[**models::ApiV2010AccountSipSipCredentialList**](api.v2010.account.sip.sip_credential_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_credential_list

> models::ListSipCredentialListResponse list_sip_credential_list(account_sid, page_size, page, page_token)
Get All Credential Lists

Get All Credential Lists

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipCredentialListResponse**](ListSipCredentialListResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sip_credential_list

> models::ApiV2010AccountSipSipCredentialList update_sip_credential_list(account_sid, sid, friendly_name)
Update a Credential List

Update a Credential List

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**sid** | **String** | The credential list Sid that uniquely identifies this resource | [required] |
**friendly_name** | **String** | A human readable descriptive text for a CredentialList, up to 64 characters long. | [required] |

### Return type

[**models::ApiV2010AccountSipSipCredentialList**](api.v2010.account.sip.sip_credential_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

