# \Api20100401IpAccessControlListApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_ip_access_control_list**](Api20100401IpAccessControlListApi.md#create_sip_ip_access_control_list) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists.json | Create a new IpAccessControlList resource
[**delete_sip_ip_access_control_list**](Api20100401IpAccessControlListApi.md#delete_sip_ip_access_control_list) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{Sid}.json | Delete an IpAccessControlList from the requested account
[**fetch_sip_ip_access_control_list**](Api20100401IpAccessControlListApi.md#fetch_sip_ip_access_control_list) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{Sid}.json | Fetch a specific instance of an IpAccessControlList
[**list_sip_ip_access_control_list**](Api20100401IpAccessControlListApi.md#list_sip_ip_access_control_list) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists.json | Retrieve a list of IpAccessControlLists that belong to the account used to make the request
[**update_sip_ip_access_control_list**](Api20100401IpAccessControlListApi.md#update_sip_ip_access_control_list) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/IpAccessControlLists/{Sid}.json | Rename an IpAccessControlList



## create_sip_ip_access_control_list

> models::ApiV2010AccountSipSipIpAccessControlList create_sip_ip_access_control_list(account_sid, friendly_name)
Create a new IpAccessControlList resource

Create a new IpAccessControlList resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**friendly_name** | **String** | A human readable descriptive text that describes the IpAccessControlList, up to 255 characters long. | [required] |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlList**](api.v2010.account.sip.sip_ip_access_control_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_ip_access_control_list

> delete_sip_ip_access_control_list(account_sid, sid)
Delete an IpAccessControlList from the requested account

Delete an IpAccessControlList from the requested account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_ip_access_control_list

> models::ApiV2010AccountSipSipIpAccessControlList fetch_sip_ip_access_control_list(account_sid, sid)
Fetch a specific instance of an IpAccessControlList

Fetch a specific instance of an IpAccessControlList

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlList**](api.v2010.account.sip.sip_ip_access_control_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_ip_access_control_list

> models::ListSipIpAccessControlListResponse list_sip_ip_access_control_list(account_sid, page_size, page, page_token)
Retrieve a list of IpAccessControlLists that belong to the account used to make the request

Retrieve a list of IpAccessControlLists that belong to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipIpAccessControlListResponse**](ListSipIpAccessControlListResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sip_ip_access_control_list

> models::ApiV2010AccountSipSipIpAccessControlList update_sip_ip_access_control_list(account_sid, sid, friendly_name)
Rename an IpAccessControlList

Rename an IpAccessControlList

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for this resource. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to udpate. | [required] |
**friendly_name** | **String** | A human readable descriptive text, up to 255 characters long. | [required] |

### Return type

[**models::ApiV2010AccountSipSipIpAccessControlList**](api.v2010.account.sip.sip_ip_access_control_list.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

